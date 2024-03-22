pub mod buckets {
    use std::{error::Error, str};

    use s3::bucket::Bucket;
    use s3::creds::Credentials;
    use s3::region::Region;
    use s3::BucketConfiguration;

    pub async fn list_objects() -> Result<(), Box<dyn Error>> {
        // 1) Instantiate the bucket client
        let bucket_name = "se-ci-artifacts";
        let region = Region::Custom {
            region: "".to_owned(),
            endpoint: "http://se-ci-storage.localdomain:9000".to_owned(),
        };
        let credentials = Credentials {
            access_key: Some("you-know-it".to_owned()),
            secret_key: Some(String::from(
                "or-you-dont",
            )),
            security_token: None,
            session_token: None,
            expiration: None,
        };

        let mut bucket =
            Bucket::new(bucket_name, region.clone(), credentials.clone())?.with_path_style();

        // 4) List bucket content
        println!("=== List bucket content ===");
        let results = bucket.head_object("/").await?;
        let objects = bucket.list("".to_owned(), Some("/".to_owned())).await?;
        for object in objects {
            println!("{:?}", object.common_prefixes.unwrap());
        }
        Ok(())
    }
}
