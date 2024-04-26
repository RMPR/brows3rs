use std::error::Error;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::serde_types::ListBucketResult;

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
