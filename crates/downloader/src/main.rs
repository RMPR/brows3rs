use clap::Parser;
use std::error::Error;

use buckets::download_artifacts_sync;
use buckets::list_all_objects;

use std::path::Path;

#[derive(Parser, Debug)]
#[command(version, about = "Download all files given a minio path", long_about = None)]
struct Args {
    #[clap(short, long)]
    artifact_path: String,

    #[clap(short, long, default_value = "artifacts")]
    destination_folder: String,
}

// #[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    // let artifact_path = download_artifacts(&args.artifact_path, &args.destination_folder).await?;
    // let objects = list_all_objects(&args.artifact_path)?;
    // for object in objects {
    //     println!("{:?}", object.prefix);
    //     for file in object.contents {
    //         println!("Download file: {:?}", file.key);
    //     }
    // }
    let artifact_path = Path::new(&args.artifact_path);
    download_artifacts_sync(&artifact_path, &args.destination_folder)?;
    Ok(())
}
