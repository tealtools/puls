mod utils;

use assert_cmd::Command;
use pulsar_compose::instance_config::InstanceConfig;
use utils::{
    check_cluster_exists, check_namespace_exists, check_tenant_exists, kill_all_docker_containers,
    rand_instance_name,
};

async fn test_pulsar_instance(instance_config: InstanceConfig) {
    let instance_config_yaml = serde_yaml::to_string(&instance_config).unwrap();
    println!("Testing instance: {instance_config_yaml}");

    let instance_config_clone = instance_config.clone();
    tokio::spawn(async move {
        let instance_name = instance_config_clone.name.clone();
        let num_clusters = instance_config_clone.num_clusters;
        let num_zookeepers = instance_config_clone.num_zookeepers;
        let num_bookies = instance_config_clone.num_bookies;
        let num_brokers = instance_config_clone.num_brokers;

        println!("Creating instance: {instance_name}");
        Command::cargo_bin("pulsar-compose")
            .unwrap()
            .arg("create")
            .arg("--name")
            .arg(instance_name.clone())
            .arg("--num-clusters")
            .arg(num_clusters.to_string())
            .arg("--num-zookeepers")
            .arg(num_zookeepers.to_string())
            .arg("--num-bookies")
            .arg(num_bookies.to_string())
            .arg("--num-brokers")
            .arg(num_brokers.to_string())
            .assert()
            .success();

        println!("Running instance: {instance_name}");
        Command::cargo_bin("pulsar-compose")
            .unwrap()
            .arg("run")
            .arg(instance_name.clone())
            .assert()
            .success();
    });

    let is_cluster_exists = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let result = check_cluster_exists(0, "cluster-0".to_string()).await;
            match result {
                Ok(_) => return true,
                Err(_) => {
                    let time_elapsed = started_at.elapsed();
                    if time_elapsed.as_secs() > 60 * 3 {
                        return false;
                    }
                }
            }
        }
    })
    .await
    .unwrap();

    assert!(is_cluster_exists);

    let is_tenant_exists = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let result = check_tenant_exists(0, "cluster-0-local".to_string()).await;
            match result {
                Ok(_) => return true,
                Err(_) => {
                    let time_elapsed = started_at.elapsed();
                    if time_elapsed.as_secs() > 60 * 3 {
                        return false;
                    }
                }
            }
        }
    })
    .await
    .unwrap();

    assert!(is_tenant_exists);

    let is_namespace_exists = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let result =
                check_namespace_exists(0, "cluster-0-local".to_string(), "default".to_string())
                    .await;
            match result {
                Ok(_) => return true,
                Err(_) => {
                    let time_elapsed = started_at.elapsed();
                    if time_elapsed.as_secs() > 60 * 3 {
                        return false;
                    }
                }
            }
        }
    })
    .await
    .unwrap();

    assert!(is_namespace_exists);

    kill_all_docker_containers();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_instance() {
    let instance_configs = vec![
        InstanceConfig {
            name: rand_instance_name(),
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 1,
            num_brokers: 1,
            num_bookies: 1,
            num_zookeepers: 1,
        },
        InstanceConfig {
            name: rand_instance_name(),
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 1,
            num_brokers: 5,
            num_bookies: 5,
            num_zookeepers: 3,
        },
        InstanceConfig {
            name: rand_instance_name(),
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 2,
            num_brokers: 1,
            num_bookies: 1,
            num_zookeepers: 1,
        },
        InstanceConfig {
            name: rand_instance_name(),
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 3,
            num_brokers: 2,
            num_bookies: 2,
            num_zookeepers: 2,
        },
    ];

    for instance_config in instance_configs {
        test_pulsar_instance(instance_config).await;
    }
}
