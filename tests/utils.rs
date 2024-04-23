use anyhow::Result;
use std::process::{Command, Stdio};
use uuid::Uuid;

pub fn rand_instance_name() -> String {
    "pulsar-".to_string() + &Uuid::new_v4().to_string()
}

pub fn cleanup_docker_resources() -> Result<()> {
    println!("Cleaning up docker resources created by tests");

    let kill_containers = "docker ps -q | xargs docker kill";
    let prune_containers = "docker container prune -f";
    let prune_volumes = "docker volume prune -a -f";
    let prune_networks = "docker network prune -f";
    let script = format!("{kill_containers} && {prune_containers} && {prune_volumes} && {prune_networks}");

    Command::new("bash")
        .stderr(Stdio::piped())
        .arg("-c")
        .arg(script)
        .spawn()?
        .wait_with_output()?;

    println!("Docker resources created by test cleaned up successfully");

    Ok(())
}

pub async fn check_cluster_exists(cluster_index: u32, cluster_name: String) -> Result<()> {
    let port: u32 = format!("{cluster_index}8080").parse().unwrap();
    let out = Command::new("pulsar-admin")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("clusters")
        .arg("list")
        .spawn()?
        .wait_with_output()?
        .stdout;

    let is_exists = String::from_utf8(out)?
        .lines()
        .any(|line| line.contains(&cluster_name));

    println!("Is cluster {cluster_index} exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Cluster {cluster_index} not exists")),
    }
}

pub async fn check_tenant_exists(cluster_index: u32, tenant: String) -> Result<()> {
    let port: u32 = format!("{cluster_index}8080").parse().unwrap();
    let out = Command::new("pulsar-admin")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("tenants")
        .arg("list")
        .spawn()?
        .wait_with_output()?
        .stdout;

    let is_exists = String::from_utf8(out)?
        .lines()
        .any(|line| line.contains(&tenant));

    println!("Is tenant cluster-{cluster_index} {tenant} exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Tenant cluster-{cluster_index} {tenant} not exists")),
    }
}

pub async fn check_namespace_exists(
    cluster_index: u32,
    tenant: String,
    namespace: String,
) -> Result<()> {
    let port: u32 = format!("{cluster_index}8080").parse().unwrap();
    let out = Command::new("pulsar-admin")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("namespaces")
        .arg("list")
        .arg(tenant.clone())
        .spawn()?
        .wait_with_output()?
        .stdout;

    let is_exists = String::from_utf8(out)?
        .lines()
        .any(|line| line.contains(&namespace));

    println!("Is namespace cluster-{cluster_index} {tenant}/{namespace} exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Namespace cluster-{cluster_index} {tenant}/{namespace} not exists")),
    }
}

pub async fn check_cluster_connection(
    cluster_index: u32,
) -> Result<()> {
    let port: u32 = format!("{cluster_index}8080").parse().unwrap();
    let is_ok = Command::new("pulsar-admin")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("brokers")
        .arg("healthcheck")
        .spawn()?.wait()?.success();

    println!("Is able to connect to cluster {cluster_index}: {is_ok}");

    match is_ok {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Unable to connect to cluster {cluster_index}")),
    }
}

