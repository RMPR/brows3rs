use futures::executor;
use minio::s3::args::{BucketExistsArgs, MakeBucketArgs, UploadObjectArgs, ListObjectsArgs};
use minio::s3::client::Client;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;

#[tokio::main]
async fn main() {
    let base_url = "http://se-ci-storage:9000".parse::<BaseUrl>().unwrap();

    let static_provider = StaticProvider::new(
        "you-know",
        "or-you-dont",
        None,
    );

    let client = Client::new(
        base_url.clone(),
        Some(Box::new(static_provider)),
        None,
        None,
    )
    .unwrap();
    let bucket_name = String::from("se-ci-artifacts");
    let exists = client.bucket_exists(&BucketExistsArgs::new(&bucket_name).unwrap()).await.unwrap();
    println!("{}", exists);
    let args = ListObjectsArgs::new(&bucket_name, &|items| { for item in items.iter() { println!("{:?}", item.name); } true },).unwrap();
    let stream = client.list_objects(&args).await;
    println!("{:?}", stream.unwrap()); 
}
