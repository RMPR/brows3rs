use std::error::Error;
use std::result;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use s3::serde_types::ListBucketResult;
use s3::serde_types::Object;

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

pub async fn list_objects(prefix: &str) -> Result<Vec<ListBucketResult>, Box<dyn Error>> {
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

#[derive(Debug, Clone)]
struct ArtifactNode {
    name: String,
    children: Vec<Box<ArtifactNode>>,
}

impl ArtifactNode {
    fn new(name: &str) -> ArtifactNode {
        ArtifactNode {
            name: name.to_string(),
            children: Vec::<Box<ArtifactNode>>::new(),
        }
    }

    fn find_child(&mut self, name: &str) -> Option<&mut ArtifactNode> {
        for c in self.children.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<ArtifactNode>,
    {
        self.children.push(Box::new(leaf.into()));
        self
    }
}

fn build_artifact_tree(node: &mut ArtifactNode, parts: &Vec<&str>, depth: usize) {
    if depth >= parts.len() {
        // Finished building the tree
        return;
    }
    let part = &parts[depth];
    let mut child_node = match node.find_child(&part) {
        Some(dir) => dir,
        None => {
            // Create a new child node and return it
            let new_node = ArtifactNode::new(&part);
            node.add_child(new_node);
            node.find_child(&part).unwrap()
        }
    };
    build_artifact_tree(&mut child_node, parts, depth + 1);
}

fn print_file(file_name: &str, depth: u32) {
    if depth == 0 {
        println!("{}", file_name);
    } else {
        println!(
            "{:indent$}{} {}",
            "",
            "└──",
            file_name,
            indent = (depth * 4) as usize
        );
    }
}

fn print_artifact_tree(node: &ArtifactNode, depth: u32) {
    print_file(&node.name, depth);
    for child in &node.children {
        print_artifact_tree(&child, depth + 1);
    }
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
    let mut temporary_folder = std::env::temp_dir();
    temporary_folder.push(destination_path);
    if !is_artifact_a_folder(artifact_path) {
        return download_artifact(artifact_path, &temporary_folder).await;
    }

    // TODO: Implement logic to visit all folders

    // let mut folders_to_visit: Vec<ListBucketResult> = Vec::new();

    // // TODO: Check that having or not having "/" at end of artifact_path works
    let bucket_results = list_objects(artifact_path).await?;
    // Create a directory, returns `io::Result<()>`
    std::fs::create_dir(temporary_folder.as_path())?;
    for bucket_result in bucket_results {
        for artifact_object in bucket_result.contents {
            // add_artifact_if_folder(artifact_path, &artifact_path, &mut to_visit_folders);
            download_artifact(&artifact_object.key, &temporary_folder).await?;
        }
    }
    // Implement the logic for recursive folder downloading
    Ok(())
}

pub fn download_artifacts_sync(
    artifact_path: &Path,
    destination_path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut temporary_folder = std::env::temp_dir();
    temporary_folder.push(destination_path);

    let objects = list_all_objects(&artifact_path.display().to_string())?;
    for object in objects {
        match object.prefix {
            None => continue,
            Some(prefix) => {
                println!("{:?}", prefix);
                let object_prefix_path = Path::new(&prefix);
                let artifact_folder = object_prefix_path.strip_prefix(artifact_path)?;
                let folder_to_create = temporary_folder.join(artifact_folder);
                std::fs::create_dir_all(&folder_to_create)?;
                println!("Downloading to folder: {:?}", &folder_to_create);
                for artifact_object in object.contents {
                    println!("Downloading file: {:?}", artifact_object.key);
                    Runtime::new()
                        .unwrap()
                        .block_on(download_artifact(&artifact_object.key, &folder_to_create))?;
                }
            }
        }
    }
    Ok(())
}
