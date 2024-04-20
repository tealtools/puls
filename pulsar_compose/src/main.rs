mod config;
mod docker_compose;

use config::config::AppConfig;
use clap::Parser;

fn main() {
    let args = AppConfig::parse();

    println!("Hello, world!");
}
