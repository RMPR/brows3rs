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
            access_key: Some("you-know".to_owned()),
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
        let results = bucket.list("success/master/sdk/commit/".to_string(), Some("/".to_string())).await?;
        for result in results {
            for item in result.contents {
                println!("key: {}", item.key);
            }
        }

        let file_to_download = "success/master/sdk/commit/2023-12-19T22:47:23/22658563/windows/Installers/ZividSetup_2.12.0-pre-alpha-1+22658563-1-preview.exe";
        // let file_to_download = "ZividSetup_2.12.0-pre-alpha-1+22658563-1-preview.exe";
        let response_data = bucket.get_object(file_to_download).await?;
        assert_eq!(response_data.status_code(), 200);

        Ok(())
    }
}
