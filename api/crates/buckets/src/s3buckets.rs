use crate::artifact_node::{build_artifact_tree, print_artifact_tree, ArtifactNode};

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::serde_types::ListBucketResult;
use s3::serde_types::Object;

use fs_more::directory::DestinationDirectoryRule;
use fs_more::directory::DirectoryMoveOptions;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use tokio::runtime::Runtime;

fn get_bucket() -> Result<Bucket, Box<dyn Error>> {
    let bucket_name =
        std::env::var("S3_BUCKET").expect("Failed to get the environment variable S3_BUCKET");
    let access_key =
        std::env::var("S3_ACCESSKEY").expect("Failed to get the environment variable S3_ACCESSKEY");
    let secret_key =
        std::env::var("S3_SECRETKEY").expect("Failed to get the environment variable S3_SECRETKEY");
    let hostname =
        std::env::var("S3_HOSTNAME").expect("Failed to get the environment variable S3_HOSTNAME");
    let region = Region::Custom {
        region: "".to_owned(),
        endpoint: format!("http://{}", hostname),
    };
    let credentials = Credentials {
        access_key: Some(access_key),
        secret_key: Some(secret_key),
        security_token: None,
        session_token: None,
        expiration: None,
    };
    let bucket =
        Bucket::new(bucket_name.as_str(), region.clone(), credentials.clone())?.with_path_style();
    Ok(bucket)
}

async fn list_objects(prefix: &str) -> Result<Vec<ListBucketResult>, Box<dyn Error>> {
    let bucket = get_bucket()?;
    let objects = bucket
        .list(String::from(prefix), Some("/".to_owned()))
        .await?;
    return Ok(objects);
}

fn find_and_append_objects(
    prefix: &str,
    mut output_objects: &mut Vec<ListBucketResult>,
) -> Result<(), Box<dyn Error>> {
    let objects_to_visit = Runtime::new().unwrap().block_on(list_objects(prefix))?;
    for object in &objects_to_visit {
        match &object.common_prefixes {
            None => continue,
            Some(common_prefixes) => {
                for common_prefix in common_prefixes {
                    find_and_append_objects(common_prefix.prefix.as_str(), &mut output_objects)?;
                }
            }
        }
    }
    for object in objects_to_visit {
        output_objects.push(object);
    }
    Ok(())
}

pub fn list_all_objects(prefix: &str) -> Result<Vec<ListBucketResult>, Box<dyn Error>> {
    let mut objects: Vec<ListBucketResult> = Vec::new();
    find_and_append_objects(prefix, &mut objects)?;
    Ok(objects)
}


pub fn list_folders_in_prefix(
    prefix: &str
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut output: Vec<String> = Vec::new();
    let objects = Runtime::new().unwrap().block_on(list_objects(prefix));
    match objects {
        Err(e) => return Err(e),
        Ok(objects) => {
            for object in objects {
                match object.common_prefixes {
                    None => continue,
                    Some(common_prefixes) => {
                        for common_prefix in common_prefixes {
                            output.push(common_prefix.prefix);
                        }
                    }
                }
            }
        }
    }
    Ok(output)
}

fn convert_to_artifact_tree(prefix: &str, objects: Vec<ListBucketResult>) -> ArtifactNode {
    let mut root = ArtifactNode::new(prefix);
    for object in objects {
        for file in object.contents {
            let file_path = file.key;
            let file_path_from_prefix = file_path.strip_prefix(prefix).unwrap();
            let parts: Vec<&str> = file_path_from_prefix.split("/").collect();
            build_artifact_tree(&mut root, &parts, 0);
        }
    }
    return root;
}

pub fn print_tree_list(prefix: &str, objects: Vec<ListBucketResult>) {
    let root = convert_to_artifact_tree(prefix, objects);
    print_artifact_tree(&root, 0);
}

pub fn print_flat_list(prefix: &str, objects: Vec<ListBucketResult>) {
    for object in objects {
        for file in object.contents {
            let file_path = file.key;
            let file_path_from_prefix = file_path.strip_prefix(prefix).unwrap();
            println!("{}", file_path_from_prefix);
        }
    }
}

async fn find_commit_hash_in(
    timestamp_folder: &str,
    commit_hash: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    let commit_folders = list_objects(timestamp_folder).await?;
    for commit_folder in commit_folders {
        match commit_folder.common_prefixes {
            None => continue,
            Some(common_prefixes) => {
                for commit in common_prefixes {
                    let commit_hash_short = commit.prefix.rsplit("/").nth(1).unwrap();
                    if commit_hash.contains(commit_hash_short) {
                        println!("Found an exact match for: {}", commit_hash_short);
                        return Ok(Some(commit.prefix));
                    }
                    if commit_hash_short.contains(commit_hash) {
                        println!("Did you mean --commit-hash {}?", commit_hash);
                        println!("Found one match {}", commit.prefix);
                        return Err("Did not find an exact match".into());
                    }
                }
            }
        }
    }
    return Ok(None);
}

pub async fn find_artifact_with_commit_hash(
    prefix: &str,
    commit_hash: &str,
) -> Result<String, Box<dyn Error>> {
    let folders_under_prefix = list_objects(prefix).await?;
    for folder_under_prefix in folders_under_prefix {
        match folder_under_prefix.common_prefixes {
            None => continue,
            Some(common_prefixes) => {
                for timestamp_folder in common_prefixes {
                    let found_commit_folder =
                        find_commit_hash_in(timestamp_folder.prefix.as_str(), commit_hash).await?;
                    match found_commit_folder {
                        None => continue,
                        Some(artifact_path) => {
                            return Ok(artifact_path);
                        }
                    }
                }
            }
        }
    }
    Err(format!("Did not find any artifact with commit hash {}", commit_hash).into())
}

async fn download_artifact(
    artifact_file: &str,
    destination_folder: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let bucket = get_bucket()?;
    let response = bucket.get_object(artifact_file).await?;
    if response.status_code() != 200 {
        return Err(format!("Failed to download artifact: {}", response.status_code()).into());
    }
    // TODO: Replace string concatenation with std::fs
    let destination = format!(
        "{}/{}",
        destination_folder.display().to_string(),
        artifact_file.rsplit("/").nth(0).unwrap()
    );
    let mut buffer = File::create(&destination)?;
    buffer.write_all(&response.as_slice())?;
    Ok(())
}

fn move_from_temp_to_dest(
    temporary_folder: &Path,
    destination_folder: &Path,
) -> Result<(), Box<dyn Error>> {
    if temporary_folder == destination_folder {
        println!(
            "Artifacts successfully downloaded to {}",
            destination_folder.display()
        );
        return Ok(());
    }

    // If destination folder exists, throw exception
    if destination_folder.exists() {
        println!(
            "Destination folder {} already exists and will be overwritten.",
            destination_folder.display()
        );
    }

    // create destination folder parent(s)
    std::fs::create_dir_all(destination_folder.parent().unwrap())?;

    match fs_more::directory::move_directory(
        temporary_folder,
        destination_folder,
        DirectoryMoveOptions {
            destination_directory_rule: DestinationDirectoryRule::AllowNonEmpty {
                colliding_file_behaviour: fs_more::file::CollidingFileBehaviour::Overwrite,
                colliding_subdirectory_behaviour:
                    fs_more::directory::CollidingSubDirectoryBehaviour::Continue,
            },
            ..Default::default()
        },
    ) {
        Ok(_) => {
            println!(
                "Artifacts successfully downloaded to {}",
                destination_folder.display()
            );
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

pub fn download_artifacts_sync(
    artifact_path_str: &str,
    destination_path_str: &str,
) -> Result<(), Box<dyn Error>> {
    let mut temporary_folder = std::env::temp_dir();
    temporary_folder.push(destination_path_str);
    let artifact_path = Path::new(artifact_path_str);

    let rt = Runtime::new().unwrap();
    let objects = list_all_objects(&artifact_path_str)?;
    for object in objects {
        match object.prefix {
            None => continue,
            Some(prefix) => {
                let object_prefix_path = Path::new(&prefix);
                let artifact_folder = object_prefix_path.strip_prefix(artifact_path)?;
                let folder_to_create = temporary_folder.join(artifact_folder);
                std::fs::create_dir_all(&folder_to_create)?;

                object
                    .contents
                    .iter()
                    .map(|artifact_object: &Object| {
                        println!("Downloading file: {:?}", &artifact_object.key);
                        rt.block_on(download_artifact(&artifact_object.key, &folder_to_create))
                    })
                    .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
            }
        }
    }
    let destination_path = Path::new(destination_path_str);
    return move_from_temp_to_dest(temporary_folder.as_path(), destination_path);
}
