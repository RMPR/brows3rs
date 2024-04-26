use clap::Parser;
use std::error::Error;

use buckets::download_artifact;

#[derive(Parser, Debug)]
#[command(version, about = "Download all files given a minio path", long_about = None)]
struct Args {
    #[clap(short, long)]
    path_to_object: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let artifact_path = download_artifact(&args.path_to_object, "artifact.file").await?;
    Ok(())
}

