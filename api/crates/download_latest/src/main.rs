use buckets::download_artifacts_sync;
use buckets::list_folders_in_prefix;

use clap::{Parser, ValueEnum};

use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(ValueEnum, Clone, Default, Debug)]
enum Architecture {
    #[default]
    Amd64,
    Arm64,
}

impl Display for Architecture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::Amd64 => write!(f, "amd64"),
            Architecture::Arm64 => write!(f, "arm64"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Download latest successful artifacts for given branch from artifact storage."
)]
struct Args {
    /// Name of the branch
    #[arg(short, long, default_value = "master")]
    branch: String,

    /// Target architecture of artifacts
    #[arg(short, long, default_value = "amd64")]
    architecture: Architecture,
}

fn os_name() -> Result<String, Box<dyn Error>> {
    let os = std::env::consts::OS;
    match os {
        "linux" => Ok("ubuntu-20.04".to_string()),
        "windows" => Ok("windows".to_string()),
        &_ => Err("Unsupported operating system".into()),
    }
}

fn bucket_to_search(branch_name: &str) -> String {
    if branch_name == "master" {
        format!("success/{}/sdk/commit/", branch_name)
    } else if branch_name.starts_with("release-") {
        format!("success/release/{}/sdk/commit/", branch_name)
    } else {
        format!("success/other/{}/sdk/commit/", branch_name)
    }
}

fn latest_artifact(branch_name: &str, architecture: Architecture) -> Result<(), Box<dyn Error>> {
    let path_to_search = bucket_to_search(&branch_name);
    let objects = list_folders_in_prefix(&path_to_search)?;
    if objects.is_empty() {
        return Err(format!(
            "No successful sdk commit artifacts found for branch {}",
            branch_name
        )
        .into());
    }
    let latest_timestamp = objects.iter().max().unwrap();
    let objects = list_folders_in_prefix(&latest_timestamp)?;
    let latest_commit_hash = objects.iter().max().unwrap();
    let artifact_platform = os_name()?;
    let artifacts_to_download = format!(
        "{}{}/{}/Release/",
        latest_commit_hash, artifact_platform, architecture
    );
    println!("Downloading artifacts from: {}", artifacts_to_download);
    download_artifacts_sync(&artifacts_to_download, "artifacts")?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let branch_name = args.branch;
    let architecture = args.architecture;
    println!(
        "Downloading latest artifacts for branch: '{}', architecture: '{}'",
        branch_name, architecture
    );
    latest_artifact(&branch_name, architecture)?;
    Ok(())
}
