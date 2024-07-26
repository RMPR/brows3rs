use buckets::download_artifacts_sync;
use buckets::list_all_objects;
use buckets::print_flat_list;
use buckets::print_tree_list;

use clap::Parser;
use clap::Subcommand;

use std::error::Error;

#[derive(Subcommand, Debug, Clone)]
pub enum ListFormat {
    #[command(about = "List all files in a flat format")]
    Flat,
    #[command(about = "List all files in a tree format")]
    Tree,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Download all files given a minio path")]
    Download {
        #[arg(short, long, default_value = "artifacts")]
        destination_folder: String,
    },
    #[command(about = "List all files given a minio path")]
    List {
        #[command(subcommand)]
        format: Option<ListFormat>,
    },
}

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Download/list all files given a minio path. Default action is to download all artifacts preserving the folder hierarchy."
)]
struct Args {
    /// The path to the artifact in minio
    artifact_path: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn print_files(prefix: &str, format: ListFormat) -> Result<(), Box<dyn Error>> {
    let objects = list_all_objects(&prefix)?;
    println!("Files in {}:", &prefix);
    match format {
        ListFormat::Flat => {
            print_flat_list(&prefix, objects);
        }
        ListFormat::Tree => {
            print_tree_list(&prefix, objects);
        }
    }
    Ok(())
}

fn strip_artifact_path_url(url: &str) -> String {
    let se_cluster = "http://se-cluster-2.localdomain:32000/se-ci-artifacts/";
    let se_ci_storage = "http://se-ci-storage:9000/minio/se-ci-artifacts/";
    if url.starts_with(se_cluster) {
        return url.replace(se_cluster, "");
    } else if url.starts_with(se_ci_storage) {
        return url.replace(se_ci_storage, "");
    }
    url.to_string()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let artifact_path = strip_artifact_path_url(&args.artifact_path);
    let command = match args.command {
        Some(c) => c,
        None => Commands::Download {
            destination_folder: "artifacts".to_string(),
        },
    };
    match command {
        Commands::Download { destination_folder } => {
            download_artifacts_sync(&artifact_path, &destination_folder)?;
        }
        Commands::List { format } => {
            let format = match format {
                Some(f) => f,
                None => ListFormat::Flat,
            };
            return print_files(&artifact_path, format);
        }
    }
    Ok(())
}
