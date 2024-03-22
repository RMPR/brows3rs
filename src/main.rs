use std::{error::Error, str};

use brows3rs::buckets::sync_list_objects;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    sync_list_objects();
}
