use crate::config::config::AppConfig;
use std::cmp::min;

pub fn generate_template(app_config: AppConfig) -> String {
    let zookeepers_per_cluster = app_config.num_zookeepers_per_cluster;
    let zookeeper_templates = (0..zookeepers_per_cluster)
        .map(|zookeeper_index| generate_zookeeper_template(app_config.clone(), zookeeper_index))
        .collect::<Vec<String>>()
        .join("\n");

    let num_clusters = app_config.num_clusters;
    let cluster_names = (0..num_clusters).map(|i| format!("cluster-{}", i));
    let cluster_templates = cluster_names
        .clone()
        .enumerate()
        .map(|(cluster_index, cluster_name)| generate_cluster_template(app_config.clone(), cluster_name, u32::try_from(cluster_index).unwrap()))
        .collect::<Vec<String>>()
        .join("\n\n");

    let volumes_template = cluster_names
        .map(|cluster_name| {
            (0..app_config.num_bookies_per_cluster)
                .map(|i| format!("████bookie-data-{cluster_name}-{i}:"))
                .collect::<Vec<String>>()
                .join("\n")
        })
        .collect::<Vec<String>>()
        .join("\n");

    let instance_name = app_config.instance_name;

    format! {"
version: '2'
services:
{zookeeper_templates}

{cluster_templates}

volumes:
{volumes_template}

networks:
████pulsar-net-{instance_name}:
████████driver: bridge
"}
    .trim()
    .replace('█', " ")
}

pub fn generate_cluster_template(app_config: AppConfig, cluster_name: String, cluster_index: u32) -> String {
    let pulsar_init_template =
        generate_pulsar_init_template(app_config.clone(), cluster_name.clone());

    let brokers_template = (0..app_config.num_brokers_per_cluster)
        .map(|i| generate_broker_template(app_config.clone(), cluster_name.clone(), i))
        .collect::<Vec<String>>()
        .join("\n");

    let bookies_template = (0..app_config.num_bookies_per_cluster)
        .map(|i| generate_bookie_template(app_config.clone(), cluster_name.clone(), i))
        .collect::<Vec<String>>()
        .join("\n");

    let pulsar_proxy_template =
        generate_pulsar_proxy_template(app_config.clone(), cluster_name.clone(), cluster_index);

    format! {"
████# BEGIN Pulsar cluster {cluster_name} definition

{pulsar_init_template}

{brokers_template}

{bookies_template}

{pulsar_proxy_template}
████# END Pulsar cluster {cluster_name} definition
"}
    .trim()
    .to_string()
}

pub fn generate_pulsar_proxy_template(app_config: AppConfig, cluster_name: String, cluster_index: u32) -> String {
    let pulsar_version = app_config.pulsar_version;
    let zookeepers_per_cluster = app_config.num_zookeepers_per_cluster;
    let depends_on_zookeeper_template = (0..zookeepers_per_cluster)
        .map(|i| format!("████████████zookeeper-{i}:\n████████████████condition: service_healthy"))
        .collect::<Vec<String>>()
        .join("\n");
    let metadata_store_url = (0..zookeepers_per_cluster)
        .map(|i| format!("zk:zookeeper-{i}:2181"))
        .collect::<Vec<String>>()
        .join(",");

    let depends_on_brokers_template = (0..app_config.num_brokers_per_cluster)
        .map(|i| format!("████████████broker-{cluster_name}-{i}:\n████████████████condition: service_healthy"))
        .collect::<Vec<String>>()
        .join("\n");

    let web_service_port = (cluster_index.to_string() + "8080").parse::<u32>().unwrap();
    let broker_service_port = (cluster_index.to_string() + "6650").parse::<u32>().unwrap();

    let instance_name = app_config.instance_name;

    format! {"
████# Pulsar proxy for cluster {cluster_name}
████pulsar-proxy-{cluster_name}:
████████image: apachepulsar/pulsar:{pulsar_version}
████████user: pulsar
████████restart: on-failure
████████command: bash -c \"bin/apply-config-from-env.py conf/proxy.conf && bin/apply-config-from-env.py conf/pulsar_env.sh && bin/pulsar proxy\"
████████ports:
████████████- {web_service_port}:8080
████████████- {broker_service_port}:6650
████████environment:
████████████- clusterName={cluster_name}
████████████- metadataStoreUrl={metadata_store_url}
████████████- configurationMetadataStoreUrl={metadata_store_url}
████████████- PULSAR_MEM=-Xms256m -Xmx256m -XX:MaxDirectMemorySize=128m
████████healthcheck:
████████████test: [\"CMD\", \"curl\", \"--fail\", \"http://127.0.0.1:8080/admin/v2/brokers/health\"]
████████████interval: 5s
████████████timeout: 5s
████████████retries: 30
████████depends_on:
{depends_on_zookeeper_template}
{depends_on_brokers_template}
████████networks:
████████████- pulsar-net-{instance_name}
"}
}

pub fn generate_zookeeper_template(app_config: AppConfig, zookeeper_index: u32) -> String {
    let pulsar_version = app_config.pulsar_version;

    let zookeepers_per_cluster = app_config.num_zookeepers_per_cluster;
    let zookeeper_servers = (0..zookeepers_per_cluster)
        .map(|i| format!("server.{i}=zookeeper-{i}:2888:3888"))
        .collect::<Vec<String>>();

    let append_zookeeper_servers = zookeeper_servers
        .iter()
        .map(|server| format!("echo \"{}\" >> /pulsar/conf/zookeeper.conf", server))
        .collect::<Vec<String>>()
        .join("; ");

    let create_my_id_if_not_exists = format!("if [ ! -f /pulsar/data/zookeeper/myid ]; then mkdir -p /pulsar/data/zookeeper && echo {zookeeper_index} > /pulsar/data/zookeeper/myid; fi");

    let instance_name = app_config.instance_name;

    format! {"
████# Zookeeper for Pulsar
████zookeeper-{zookeeper_index}:
████████image: apachepulsar/pulsar:{pulsar_version}
████████user: pulsar
████████restart: on-failure
████████command: bash -c \"bin/apply-config-from-env.py conf/zookeeper.conf && bin/apply-config-from-env.py conf/pulsar_env.sh && {append_zookeeper_servers} && {create_my_id_if_not_exists} && exec bin/pulsar zookeeper\"
████████environment:
████████████- PULSAR_MEM=\"-Xms256m -Xmx256m -XX:MaxDirectMemorySize=256m\"
████████healthcheck:
████████████test: [\"CMD\", \"bin/pulsar-zookeeper-ruok.sh\"]
████████████interval: 5s
████████████timeout: 5s
████████████retries: 10
████████networks:
████████████- pulsar-net-{instance_name}
"}
    .trim()
    .to_string()
}

pub fn generate_pulsar_init_template(app_config: AppConfig, cluster_name: String) -> String {
    let pulsar_version = app_config.pulsar_version;
    let web_service_url = "http://broker-{cluster_name}:8080";
    let broker_service_url = "pulsar://broker-{custer_name}:6650";
    let zookeepers_per_cluster = app_config.num_zookeepers_per_cluster;
    let depends_on_zookeeper_template = (0..zookeepers_per_cluster)
        .map(|i| format!("████████████zookeeper-{i}:\n████████████████condition: service_healthy"))
        .collect::<Vec<String>>()
        .join("\n");

    let instance_name = app_config.instance_name;

    format! {"
████# Pulsar init job for cluster {cluster_name}
████pulsar-init-{cluster_name}:
████████image: apachepulsar/pulsar:{pulsar_version}
████████user: pulsar
████████command: bash -c \"bin/pulsar initialize-cluster-metadata --cluster {cluster_name} --metadata-store zk:zookeeper-0:2181/{cluster_name} --configuration-metadata-store zk:zookeeper-0:2181/{cluster_name} --web-service-url {web_service_url} --broker-service-url {broker_service_url}\"
████████depends_on:
{depends_on_zookeeper_template}
████████networks:
████████████- pulsar-net-{instance_name}
"}.trim().to_string()
}

pub fn generate_broker_template(
    app_config: AppConfig,
    cluster_name: String,
    broker_index: u32,
) -> String {
    let pulsar_version = app_config.pulsar_version;
    let managed_ledger_default_ensemble_size = min(app_config.num_bookies_per_cluster, 3);
    let managed_ledger_default_write_quorum = min(app_config.num_bookies_per_cluster, 3);
    let managed_ledger_default_ack_quorum = min(app_config.num_bookies_per_cluster, 3);
    let zookeepers_per_cluster = app_config.num_zookeepers_per_cluster;
    let depends_on_zookeeper_template = (0..zookeepers_per_cluster)
        .map(|i| format!("████████████zookeeper-{i}:\n████████████████condition: service_healthy"))
        .collect::<Vec<String>>()
        .join("\n");

    let depends_on_bookies_template = (0..app_config.num_bookies_per_cluster)
        .map(|i| format!("████████████bookie-{cluster_name}-{i}:\n████████████████condition: service_healthy"))
        .collect::<Vec<String>>()
        .join("\n");

    let metadata_store_url = (0..zookeepers_per_cluster)
        .map(|i| format!("zk:zookeeper-{i}:2181"))
        .collect::<Vec<String>>()
        .join(",");

    let instance_name = app_config.instance_name;

    format! {"
████# Pulsar broker for cluster {cluster_name}
████broker-{cluster_name}-{broker_index}:
████████image: apachepulsar/pulsar:{pulsar_version}
████████user: pulsar
████████restart: on-failure
████████environment:
████████████- clusterName={cluster_name}
████████████- metadataStoreUrl={metadata_store_url}
████████████- configurationMetadataStoreUrl={metadata_store_url}
████████████- managedLedgerDefaultEnsembleSize={managed_ledger_default_ensemble_size}
████████████- managedLedgerDefaultWriteQuorum={managed_ledger_default_write_quorum}
████████████- managedLedgerDefaultAckQuorum={managed_ledger_default_ack_quorum}
████████████- PULSAR_MEM=-Xms512m -Xmx512m -XX:MaxDirectMemorySize=256m
████████command: bash -c \"bin/apply-config-from-env.py conf/broker.conf && exec bin/pulsar broker\"
████████healthcheck:
████████████test: [\"CMD\", \"curl\", \"--fail\", \"http://127.0.0.1:8080/admin/v2/brokers/health\"]
████████████interval: 10s
████████████timeout: 5s
████████████retries: 20
████████depends_on:
{depends_on_zookeeper_template}
{depends_on_bookies_template}
████████networks:
████████████- pulsar-net-{instance_name}
"}
    .trim()
    .to_string()
}

pub fn generate_bookie_template(
    app_config: AppConfig,
    cluster_name: String,
    bookie_index: u32,
) -> String {
    let pulsar_version = app_config.pulsar_version;
    let depends_on_bookie: Option<u32> = if bookie_index == 0 {
        None
    } else {
        Some(bookie_index - 1)
    };

    let depends_on_bookies_template = match depends_on_bookie {
        Some(i) => format!(
            "████████████bookie-{cluster_name}-{i}:\n████████████████condition: service_healthy"
        ),
        None => "".to_string(),
    };

    let zookeepers_per_cluster = app_config.num_zookeepers_per_cluster;

    let depends_on_zookeeper_template = (0..zookeepers_per_cluster)
        .map(|i| format!("████████████zookeeper-{i}:\n████████████████condition: service_healthy"))
        .collect::<Vec<String>>()
        .join("\n");

    let metadata_service_uri = format!(
        "zk://{}",
        (0..zookeepers_per_cluster)
            .map(|i| format!("zookeeper-{i}:2181"))
            .collect::<Vec<String>>()
            .join(";")
    ) + "/ledgers";

    let instance_name = app_config.instance_name;

    format! {"
████# Bookie for cluster {cluster_name}
████bookie-{cluster_name}-{bookie_index}:
████████image: apachepulsar/pulsar:{pulsar_version}
████████user: pulsar
████████restart: on-failure
████████environment:
████████████- clusterName={cluster_name}
████████████- metadataServiceUri={metadata_service_uri}
████████████- useHostNameAsBookieID=true
████████████- BOOKIE_MEM=-Xms512m -Xmx512m -XX:MaxDirectMemorySize=256m
████████████- dbStorage_writeCacheMaxSizeMb=32
████████████- dbStorage_readAheadCacheMaxSizeMb=32
████████depends_on:
████████████pulsar-init-{cluster_name}:
████████████████condition: service_completed_successfully
{depends_on_zookeeper_template}
{depends_on_bookies_template}
████████command: bash -c \"set -e; bin/apply-config-from-env.py conf/bookkeeper.conf; bin/apply-config-from-env.py conf/pulsar_env.sh; if bin/bookkeeper shell whatisinstanceid; then echo bookkeeper_cluster_already_initialized; else echo init_new_bookkeeper_cluster_start; bin/bookkeeper shell initnewcluster; echo init_new_bookkeeper_cluster_end; fi; exec bin/pulsar bookie\"
████████volumes:
████████████- bookie-data-{cluster_name}-{bookie_index}:/pulsar/data
████████healthcheck:
████████████test: [\"CMD\", \"/pulsar/bin/bookkeeper\", \"shell\", \"bookiesanity\"]
████████████interval: 10s
████████████timeout: 30s
████████████retries: 30
████████networks:
████████████- pulsar-net-{instance_name}
"}.trim().to_string()
}
