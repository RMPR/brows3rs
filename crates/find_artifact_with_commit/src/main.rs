use clap::Parser;
use std::error::Error;

use buckets::find_artifact_with_commit_hash;

#[derive(Parser, Debug)]
#[command(version, about = "Retrieve the artifacts path from the commit hash", long_about = None)]
struct Args {
    #[clap(short, long)]
    release: String,

    #[clap(short, long)]
    commit_hash: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let release = args.release;
    let artifact_path = find_artifact_with_commit_hash(
        format!("success/release/release-sdk-{}/sdk/commit/", release).as_str(),
        args.commit_hash.as_str(),
    )
    .await?;
    println!("{}", artifact_path);
    Ok(())
}
