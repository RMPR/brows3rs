use buckets::download_artifacts_sync;
use buckets::list_all_objects;
use buckets::print_flat_list;
use buckets::print_tree_list;
use clap::Parser;
use clap::Subcommand;
use std::error::Error;

use std::path::Path;

#[derive(Subcommand, Debug, Clone)]
pub enum ListFormat {
    #[command(name = "flat", about = "List all files in a flat format")]
    Flat,
    #[command(name = "tree", about = "List all files in a tree format")]
    Tree,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Download {
        #[arg(short, long, default_value = "artifacts")]
        destination_folder: String,
    },
    List {
        #[command(subcommand)]
        format: ListFormat,
    },
}

#[derive(Parser, Debug)]
#[command(version, about = "Download all files given a minio path", long_about = None)]
struct Args {
    #[arg(short, long)]
    artifact_path: String,

    #[command(subcommand)]
    command: Commands,
}

fn print_files(prefix: &str, format: &ListFormat) -> Result<(), Box<dyn Error>> {
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

// #[tokio::main]
fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let artifact_path = Path::new(&args.artifact_path);
    match &args.command {
        Commands::Download { destination_folder } => {
            download_artifacts_sync(&artifact_path, destination_folder)?;
        }
        Commands::List { format } => {
            let artifact_path_str = artifact_path.to_str().unwrap();
            println!("Files in {}:", &artifact_path_str);
            return print_files(&artifact_path_str, format);
        }
    }
    Ok(())
}
