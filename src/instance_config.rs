use clap::Parser;
use serde::{Deserialize, Serialize};

const DEFAULT_INSTANCE_NAME: &str = "default";
const DEFAULT_PULSAR_VERSION: &str = "3.2.2";
const DEFAULT_NUM_CLUSTERS: &str = "1";
const DEFAULT_NUM_BROKERS: &str = "1";
const DEFAULT_NUM_BOOKIES: &str = "1";
const DEFAULT_NUM_ZOOKEEPERS: &str = "1";

#[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[command(version, about, long_about = None)]
pub struct InstanceConfig {
    #[arg(long, default_value = DEFAULT_INSTANCE_NAME)]
    pub name: String,

    #[arg(long, default_value = DEFAULT_PULSAR_VERSION)]
    pub pulsar_version: String,

    #[arg(long, default_value = DEFAULT_NUM_CLUSTERS)]
    pub num_clusters: u32,

    #[arg(long, default_value = DEFAULT_NUM_BROKERS)]
    pub num_brokers: u32,

    #[arg(long, default_value = DEFAULT_NUM_BOOKIES)]
    pub num_bookies: u32,

    #[arg(long, default_value = DEFAULT_NUM_ZOOKEEPERS)]
    pub num_zookeepers: u32,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        InstanceConfig {
            name: DEFAULT_INSTANCE_NAME.to_string(),
            pulsar_version: DEFAULT_PULSAR_VERSION.to_string(),
            num_clusters: DEFAULT_NUM_CLUSTERS.parse().unwrap(),
            num_brokers: DEFAULT_NUM_BROKERS.parse().unwrap(),
            num_bookies: DEFAULT_NUM_BOOKIES.parse().unwrap(),
            num_zookeepers: DEFAULT_NUM_ZOOKEEPERS.parse().unwrap(),
        }
    }
}
