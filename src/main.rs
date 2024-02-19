use std::{error::Error, str};

use crate::brows3r::buckets::list_objects;

pub mod brows3r;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    list_objects().await
}
