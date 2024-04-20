mod config;
use config::config::Args;
use clap::Parser;

fn main() {
    let args = Args::parse();

    println!("Hello, world!");
}
