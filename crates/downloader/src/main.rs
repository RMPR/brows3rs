use clap::Parser;
use std::error::Error;

use buckets::download_artifacts;

#[derive(Parser, Debug)]
#[command(version, about = "Download all files given a minio path", long_about = None)]
struct Args {
    #[clap(short, long)]
    artifact_path: String,

    #[clap(short, long, default_value = "artifacts")]
    destination_folder: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let artifact_path = download_artifacts(&args.artifact_path, &args.destination_folder).await?;
    Ok(())
}

