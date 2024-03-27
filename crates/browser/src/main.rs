use std::{error::Error};

use buckets::list_objects;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    list_objects().await
}
