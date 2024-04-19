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

pub async fn find_artifacts_path(
    prefix: &str,
    commit_hash: &str,
) -> Result<String, Box<dyn Error>> {
    let folders_under_prefix = list_objects(prefix).await?;
    for artifact in folders_under_prefix {
        match artifact.common_prefixes {
            None => continue,
            Some(common_prefixes) => {
                for time_stamp_folders in common_prefixes {
                    let commit_folders = list_objects(time_stamp_folders.prefix.as_str()).await?;
                    for commit_folder in commit_folders {
                        match commit_folder.common_prefixes {
                            None => continue,
                            Some(common_prefixes) => {
                                for commit in common_prefixes {
                                    let commit_hash_short =
                                        commit.prefix.rsplit("/").nth(1).unwrap();
                                    if commit_hash.contains(commit_hash_short) {
                                        println!("Aha found it: {:?}", commit.prefix);
                                        return Ok(commit.prefix);
                                    }
                                    if commit_hash_short.contains(commit_hash) {
                                        println!("Did you mean --commit-hash {:?}?", commit_hash);
                                        return Err(
                                            format!("Found first match {}", commit.prefix).into()
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Err(format!("Did not find any artifact with commit hash {}", commit_hash).into())
}
