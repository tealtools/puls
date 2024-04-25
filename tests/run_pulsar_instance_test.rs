mod utils;

use anyhow::Result;
use assert_cmd::cargo::CommandCargoExt;
use puls::instance_config::InstanceConfig;
use std::process::Command;
use tokio::time::{sleep, Duration};
use utils::{
    check_cluster_connection, check_cluster_exists, check_namespace_exists, check_tenant_exists,
    cleanup_docker_resources, rand_instance_name,
};

async fn test_start_pulsar_instance(
    instance_name: String,
    instance_config: InstanceConfig,
) -> Result<()> {
    let instance_config_yaml = serde_yaml::to_string(&instance_config)?;
    println!("Testing instance: {instance_config_yaml}");

    let instance_config_clone = instance_config.clone();
    tokio::spawn(async move {
        let num_clusters = instance_config_clone.num_clusters;
        let num_zookeepers = instance_config_clone.num_zookeepers;
        let num_bookies = instance_config_clone.num_bookies;
        let num_brokers = instance_config_clone.num_brokers;

        println!("Creating instance: {instance_name}");
        let exit_status = Command::cargo_bin("puls")
            .unwrap()
            .arg("create")
            .arg("--num-clusters")
            .arg(num_clusters.to_string())
            .arg("--num-zookeepers")
            .arg(num_zookeepers.to_string())
            .arg("--num-bookies")
            .arg(num_bookies.to_string())
            .arg("--num-brokers")
            .arg(num_brokers.to_string())
            .arg(instance_name.clone())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        assert!(exit_status.success());

        println!("Starting instance: {instance_name}");
        let exit_status = Command::cargo_bin("puls")
            .unwrap()
            .arg("start")
            .arg(instance_name.clone())
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

        assert!(exit_status.success());
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
                    check_namespace_exists(i, format!("cluster-{i}-local"), "default".to_string())
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

    cleanup_docker_resources()?;

    let wait_for_seconds = 10;
    println!("Waiting for {wait_for_seconds} seconds before running test next test to fix test flakiness");
    // tokio::time::sleep(Duration::from_secs(wait_for_seconds)).await;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_start_pulsar_instance_c1_br1_bk1_zk1() {
    let instance_name = rand_instance_name();
    test_start_pulsar_instance(
        instance_name,
        InstanceConfig {
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 1,
            num_brokers: 1,
            num_bookies: 1,
            num_zookeepers: 1,
            with_dekaf: false,
        },
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_start_pulsar_instance_c2_br1_bk1_zk1() {
    let instance_name = rand_instance_name();
    test_start_pulsar_instance(
        instance_name,
        InstanceConfig {
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 2,
            num_brokers: 1,
            num_bookies: 1,
            num_zookeepers: 1,
            with_dekaf: false,
        },
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_start_pulsar_instance_c3_br1_bk1_zk1() {
    let instance_name = rand_instance_name();
    test_start_pulsar_instance(
        instance_name,
        InstanceConfig {
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 3,
            num_brokers: 1,
            num_bookies: 1,
            num_zookeepers: 1,
            with_dekaf: false,
        },
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_start_pulsar_instance_c1_br3_bk3_zk3() {
    let instance_name = rand_instance_name();
    test_start_pulsar_instance(
        instance_name,
        InstanceConfig {
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 1,
            num_brokers: 3,
            num_bookies: 3,
            num_zookeepers: 3,
            with_dekaf: false,
        },
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_start_pulsar_instance_c1_br2_bk2_zk1() {
    let instance_name = rand_instance_name();
    test_start_pulsar_instance(
        instance_name,
        InstanceConfig {
            pulsar_version: "3.2.2".to_string(),
            num_clusters: 1,
            num_brokers: 2,
            num_bookies: 2,
            num_zookeepers: 1,
            with_dekaf: false,
        },
    )
    .await
    .unwrap();
}

async fn test_restart_pulsar_instance(
    instance_name: String,
    instance_config: InstanceConfig,
) -> Result<()> {
    let num_clusters = instance_config.num_clusters;
    let num_zookeepers = instance_config.num_zookeepers;
    let num_bookies = instance_config.num_bookies;
    let num_brokers = instance_config.num_brokers;

    println!("Creating instance: {instance_name}");
    let exit_status = Command::cargo_bin("puls")?
        .arg("create")
        .arg("--num-clusters")
        .arg(num_clusters.to_string())
        .arg("--num-zookeepers")
        .arg(num_zookeepers.to_string())
        .arg("--num-bookies")
        .arg(num_bookies.to_string())
        .arg("--num-brokers")
        .arg(num_brokers.to_string())
        .arg(instance_name.clone())
        .spawn()?
        .wait()?;

    assert!(exit_status.success());

    fn start_instance(instance_name: String) -> Result<()> {
        let exit_status = Command::cargo_bin("puls")?
            .arg("start")
            .arg(instance_name.clone())
            .spawn()?
            .wait()?;

        assert!(exit_status.success());

        let exit_status = Command::cargo_bin("puls")?
            .arg("stop")
            .arg(instance_name)
            .spawn()?
            .wait()?;

        assert!(exit_status.success());

        Ok(())
    }

    println!("Starting instance the first time: {instance_name}");

    start_instance(instance_name.clone())?;

    println!("Instance was successfully create and then stopped without purging it's data: {instance_name}");
    println!("Starting the same instance the second time: {instance_name}");

    start_instance(instance_name.clone())?;

    println!("Instance was successfully create and then stopped without purging it's data: {instance_name}");
    println!("Starting the same instance the third time: {instance_name}");

    start_instance(instance_name.clone())?;
    cleanup_docker_resources()?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_restart_pulsar_instance_c1_br1_bk1_zk1() {
    let instance_name = rand_instance_name();
    let instance_config = InstanceConfig {
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 1,
        num_brokers: 1,
        num_bookies: 1,
        num_zookeepers: 1,
        with_dekaf: false,
    };

    test_restart_pulsar_instance(instance_name, instance_config)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_restart_pulsar_instance_c2_br2_bk2_zk3() {
    let instance_name = rand_instance_name();
    let instance_config = InstanceConfig {
        pulsar_version: "3.2.2".to_string(),
        num_clusters: 2,
        num_brokers: 2,
        num_bookies: 2,
        num_zookeepers: 3,
        with_dekaf: false,
    };

    test_restart_pulsar_instance(instance_name, instance_config)
        .await
        .unwrap();
}
