use std::error::Error;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::serde_types::ListBucketResult;

use std::io::prelude::*;
use std::fs::File;

fn get_bucket() -> Result<Bucket, Box<dyn Error>> {
    let bucket_name = std::env::var("S3_BUCKET")?;
    let access_key = std::env::var("S3_ACCESSKEY")?;
    let secret_key = std::env::var("S3_SECRETKEY")?;
    let hostname = std::env::var("S3_HOSTNAME")?;
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

pub async fn list_objects(prefix: &str) -> Result<Vec<ListBucketResult>, Box<dyn Error>> {
    let bucket = get_bucket()?;
    let objects = bucket
        .list(String::from(prefix), Some("/".to_owned()))
        .await?;
    return Ok(objects);
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
    destination_folder: &str,
) -> Result<(), Box<dyn Error>> {
    let bucket = get_bucket()?;
    let response = bucket.get_object(artifact_file).await?;
    if (response.status_code() != 200)
    {
        return Err(format!("Failed to download artifact: {}", response.status_code()).into());
    }
    // TODO: Replace string concatenation with std::fs
    let destination = format!("{}/{}", destination_folder, artifact_file.rsplit("/").nth(0).unwrap());
    let mut buffer = File::create(&destination)?;
    buffer.write_all(&response.as_slice())?;
    println!("Downloaded artifact to {}", &destination);
    Ok(())
}

// fn is_folder(object: &ListBucketResult) -> bool {
//     object.key.ends_with("/")
// }

fn is_artifact_a_folder(artifact_path: &str) -> bool {
    // TODO: This enforces that if downloading folder, it must end with "/"
    // Update logic to support downloading folders without "/" at end
    return artifact_path.ends_with("/");
}

// fn add_artifact_if_folder(
//     artifact_path: &str,
//     object: &ListBucketResult,
//     artifact_paths: &mut Vec<String>,
// ) 
// {
//     if is_folder(object) {
//         artifact_paths.push(format!("{}/{}", artifact_path, object.key));
//     }
// }

pub async fn download_artifacts(
    artifact_path: &str,
    destination_path: &str,
) -> Result<(), Box<dyn Error>> {
    if (!is_artifact_a_folder(artifact_path)) {
        return download_artifact(artifact_path, destination_path).await;
    }

    // TODO: Implement logic to visit all folders

    // let mut to_visit_folders = Vec<ListBucketResult>::new();

    // // TODO: Check that having or not having "/" at end of artifact_path works
    // let artifact_paths = list_objects(artifact_path).await?;
    // for artifact_path in artifact_paths {
    //     add_artifact_if_folder(artifact_path, &artifact_path, &mut to_visit_folders);
    //     download_artifact(&artifact_path, destination_path).await?;
    // }
    Ok(())
}
