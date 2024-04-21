use clap::{command, Parser};
use std::convert::From;
use serde::{Serialize, Deserialize};

#[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[command(version, about, long_about = None)]
pub struct PulsarInstanceConfig {
    pub name: String,

    #[arg(long, default_value = "3.2.2")]
    pub pulsar_version: String,

    #[arg(long, default_value = "1")]
    pub num_clusters: u32,

    #[arg(long, default_value = "3")]
    pub num_brokers: u32,

    #[arg(long, default_value = "3")]
    pub num_bookies: u32,

    #[arg(long, default_value = "3")]
    pub num_zookeepers: u32,
}
