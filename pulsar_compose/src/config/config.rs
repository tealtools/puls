use clap::{command, Parser};
use std::convert::From;
use strum::EnumString;

#[derive(Clone, Debug)]
pub struct PulsarInstance {
    pub clusters: Vec<PulsarCluster>,
}

#[derive(Clone, Debug)]
pub struct PulsarCluster {
    pub name: String,
    pub oxias: Vec<Oxia>,
    pub bookies: Vec<Bookie>,
    pub brokers: Vec<PulsarBroker>,
}

#[derive(Clone, Debug)]
pub struct PulsarBroker {}

#[derive(Clone, Debug)]
pub struct Bookie {}

#[derive(Clone, Debug)]
pub struct Oxia {}

#[derive(EnumString, Clone, Debug)]
pub enum BasicConfigResources {
    Nano,
    Micro,
    Small,
    Medium,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "default")]
    pub instance_name: String,

    #[arg(short, long)]
    pub num_clusters: u32,

    #[arg(short, long)]
    pub num_brokers_per_cluster: u32,

    #[arg(short, long)]
    pub num_bookies_per_cluster: u32,

    #[arg(short, long)]
    pub num_oxias_per_cluster: u32,

    #[arg(short, long)]
    pub resources: BasicConfigResources,
}
