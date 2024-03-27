use std::error::Error;

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

pub async fn list_objects() -> Result<(), Box<dyn Error>> {
    // 1) Instantiate the bucket client
    // read the bucket name from the environment
    let bucket_name = std::env::var("S3_BUCKET")?;
    let access_key = std::env::var("S3_ACCESSKEY")?;
    let secret_key = std::env::var("S3_SECRETKEY")?;
    let region = Region::Custom {
        region: "".to_owned(),
        endpoint: "http://se-ci-storage.localdomain:9000".to_owned(),
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

    let mut num_objects = 0;

    // 4) List bucket content
    println!("=== List bucket content ===");
    let _results = bucket.head_object("/").await?;
    let objects = bucket.list("".to_owned(), Some("/".to_owned())).await?;
    for object in objects {
        println!("{:?}", object.common_prefixes.unwrap());
        num_objects += 1;
    }

    if num_objects == 0 {
        panic!("Empty");
    }

    Ok(())
}
