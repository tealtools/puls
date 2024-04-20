mod config;
mod docker_compose;

use config::config::AppConfig;
use clap::Parser;
use docker_compose::docker_compose::generate_template;

fn main() {
    let config = AppConfig::parse();
    let template = generate_template(config);

    println!("{}", template)
}
