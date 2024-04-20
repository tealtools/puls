use std::sync::Arc;

use crate::config::config::{AppConfig};
use indoc::formatdoc;
use std::cmp::min

pub fn generate_template(config: AppConfig) {}

pub fn generate_pulsar_init_template(app_config: AppConfig, cluster_name: String) -> String {
    let instance_name = app_config.instance_name;
    let pulsar_version = app_config.pulsar_version;
    let web_service_url = "http://broker-{cluster_name}:8080";
    let broker_service_url = "pulsar://broker-{custer_name}:6650";

    formatdoc! {"
        pulsar-init-{cluster_name}:
            image: apachepulsar/pulsar:{pulsar_version}
            entrypoint: bash
            command: -c \"bin/pulsar initialize-cluster-metadata --cluster {cluster_name} --metadata-store:zk:zookeeper:2181 --configuration-metadata-store=zk:zookeeper:2181 --web-service-url={web_service_url} --broker-service-url={broker_service_url}\"
    "}
}

pub fn generate_broker_template(app_config: AppConfig, cluster_name: String) -> String {
    let pulsar_version = app_config.pulsar_version;
    let num_replicas = app_config.num_brokers_per_cluster;
    let managed_ledger_default_ensemble_size = min(app_config.num_bookies_per_cluster, 3);
    let managed_ledger_default_write_quorum = min(app_config.num_bookies_per_cluster, 3);
    let managed_ledger_default_ack_quorum = min(app_config.num_bookies_per_cluster, 3);

    formatdoc! {"
        broker-{cluster_name}:
            image: apachepulsar/pulsar:{pulsar_version}
            environment:
                - clusterName={cluster_name}
                - metadataStoreUrl=\"zk://zookeeper:2181\"
                - configurationMetadataStoreUrl=\"zk://zookeeper:2181\"
                - managedLedgerDefaultEnsembleSize={managed_ledger_default_ensemble_size}
                - managedLedgerDefaultWriteQuorum={managed_ledger_default_write_quorum}
                - managedLedgerDefaultAckQuorum={managed_ledger_default_ack_quorum}
                - advertisedAddress=broker
            depends_on:
                zookeeper:
                    condition: service_healthy
                bookie-{cluster_name}:
                    condition: service_healthy
            command: bash -c \"bin/apply-config-from-env.py conf/broker.conf && exec bin/pulsar broker\"
            deploy:
                mode: replicated
                replicas: {num_replicas}
                endpoint_mode: dnsrr
    "}
}

pub fn generate_bookie_template(app_config: AppConfig, cluster_name: String) -> String {
    let pulsar_version = app_config.pulsar_version;
    let num_replicas = app_config.num_bookies_per_cluster;

    formatdoc! {"
        bookie-{cluster_name}:
            image: apachepulsar/pulsar:{pulsar_version}
            environment:
                - clusterName={cluster_name}
                - metadataServiceUri=\"zk://zookeeper:2181/ledgers\"
                - advertisedAddress=bookie
            depends_on:
                zookeeper:
                    condition: service_healthy
                pulsar-init-{cluster_name}:
                    condition: service_completed_successfully
            command: bash -c \"bin/apply-config-from-env.py conf/bookie.conf && exec bin/pulsar bookie\"
            deploy:
                mode: replicated
                replicas: {num_replicas}
                endpoint_mode: dnsrr
    "}
}
