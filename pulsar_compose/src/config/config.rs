use clap::{command, Parser};
use std::convert::From;
use strum::EnumString;

#[derive(EnumString, Clone, Debug)]
pub enum BasicConfigResources {
    Nano,
    Micro,
    Small,
    Medium,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct AppConfig {
    #[arg(short, long, default_value = "default")]
    pub instance_name: String,

    #[arg(short, long, default_value = "3.2.2")]
    pub pulsar_version: String,

    #[arg(short, long, default_value = "1")]
    pub num_clusters: u32,

    #[arg(short, long, default_value = "3")]
    pub num_brokers_per_cluster: u32,

    #[arg(short, long, default_value = "3")]
    pub num_bookies_per_cluster: u32,

    #[arg(short, long, default_value = "Small")]
    pub resources: BasicConfigResources,
}
