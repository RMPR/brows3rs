use clap::Parser;
use std::error::Error;

use browser_api::start_server;

fn main() {
    start_server(8080);
}
