mod utils;

use anyhow::Result;
use assert_cmd::cargo::CommandCargoExt;
use pulsar_compose::instance_config::InstanceConfig;
use std::process::Command;
use tokio::time::{sleep, Duration};
use utils::{
    check_cluster_connection, check_cluster_exists, check_namespace_exists, check_tenant_exists,
    kill_all_docker_containers, rand_instance_name,
};

async fn test_pulsar_instance(instance_config: InstanceConfig) -> Result<()> {
    let instance_config_yaml = serde_yaml::to_string(&instance_config)?;
    println!("Testing instance: {instance_config_yaml}");

    let instance_config_clone = instance_config.clone();
    tokio::spawn(async move {
        let instance_name = instance_config_clone.name.clone();
        let num_clusters = instance_config_clone.num_clusters;
        let num_zookeepers = instance_config_clone.num_zookeepers;
        let num_bookies = instance_config_clone.num_bookies;
        let num_brokers = instance_config_clone.num_brokers;

        println!("Creating instance: {instance_name}");
        let exit_status = Command::cargo_bin("pulsar-compose")
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
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        assert!(exit_status.success());

        println!("Running instance: {instance_name}");
        Command::cargo_bin("pulsar-compose")
            .unwrap()
            .arg("run")
            .arg(instance_name.clone())
            .spawn()
            .unwrap()
            .wait()
            .unwrap()
    });

    let is_able_to_connect_to_all_clusters = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let futures = (0..instance_config.num_clusters)
                .map(check_cluster_connection)
                .collect::<Vec<_>>();
            let results = futures::future::join_all(futures).await;

            match results.iter().all(|r| r.is_ok()) {
                true => return true,
                false => {
                    let time_elapsed = started_at.elapsed();
                    sleep(Duration::from_secs(10)).await;

                    if time_elapsed.as_secs() > 60 * 3 {
                        return false;
                    }
                }
            }
        }
    })
    .await?;

    assert!(is_able_to_connect_to_all_clusters);

    let are_clusters_created = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let futures = (0..instance_config.num_clusters)
                .map(|i| check_cluster_exists(i, format!("cluster-{i}")))
                .collect::<Vec<_>>();
            let results = futures::future::join_all(futures).await;

            match results.iter().all(|r| r.is_ok()) {
                true => return true,
                false => {
                    let time_elapsed = started_at.elapsed();
                    sleep(Duration::from_secs(10)).await;

                    if time_elapsed.as_secs() > 60 * 3 {
                        return false;
                    }
                }
            }
        }
    })
    .await?;

    assert!(are_clusters_created);

    let are_tenants_created = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let futures = (0..instance_config.num_clusters)
                .map(|i| check_tenant_exists(i, format!("cluster-{i}-local")))
                .collect::<Vec<_>>();
            let results = futures::future::join_all(futures).await;

            match results.iter().all(|r| r.is_ok()) {
                true => return true,
                false => {
                    let time_elapsed = started_at.elapsed();
                    sleep(Duration::from_secs(10)).await;

                    if time_elapsed.as_secs() > 60 * 3 {
                        return false;
                    }
                }
            }
        }
    })
    .await?;

    assert!(are_tenants_created);

    let is_namespace_exists = tokio::spawn(async move {
        let started_at = std::time::Instant::now();
        loop {
            let futures = (0..instance_config.num_clusters)
                .map(|i| {
                    check_namespace_exists(
                        i,
                        format!("cluster-{i}-local"),
                        "default".to_string(),
                    )
                })
                .collect::<Vec<_>>();

            let results = futures::future::join_all(futures).await;

            match results.iter().all(|r| r.is_ok()) {
                true => return true,
                false => {
                    let time_elapsed = started_at.elapsed();
                    sleep(Duration::from_secs(10)).await;

                    if time_elapsed.as_secs() > 60 * 10 {
                        return false;
                    }
                }
            }
        }
    })
    .await?;

    assert!(is_namespace_exists);

    kill_all_docker_containers()
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_pulsar_instance_c1_br1_bo1_zo1() {
    test_pulsar_instance(InstanceConfig {
        name: rand_instance_name(),
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 1,
        num_brokers: 1,
        num_bookies: 1,
        num_zookeepers: 1,
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_pulsar_instance_c2_br1_bo1_zo1() {
    test_pulsar_instance(InstanceConfig {
        name: rand_instance_name(),
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 2,
        num_brokers: 1,
        num_bookies: 1,
        num_zookeepers: 1,
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_pulsar_instance_c3_br1_bo1_zo1() {
    test_pulsar_instance(InstanceConfig {
        name: rand_instance_name(),
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 3,
        num_brokers: 1,
        num_bookies: 1,
        num_zookeepers: 1,
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_pulsar_instance_c1_br3_bo3_zo3() {
    test_pulsar_instance(InstanceConfig {
        name: rand_instance_name(),
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 1,
        num_brokers: 3,
        num_bookies: 3,
        num_zookeepers: 3,
    })
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_run_pulsar_instance_c1_br2_bo2_zo1() {
    test_pulsar_instance(InstanceConfig {
        name: rand_instance_name(),
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 1,
        num_brokers: 2,
        num_bookies: 2,
        num_zookeepers: 1,
    })
    .await
    .unwrap();
}
