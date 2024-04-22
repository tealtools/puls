use anyhow::Result;
use std::process::{Command, Stdio};
use uuid::Uuid;

pub fn rand_instance_name() -> String {
    "pulsar-".to_string() + &Uuid::new_v4().to_string()
}

pub fn kill_all_docker_containers() -> Result<()> {
    println!("Killing all docker containers");

    Command::new("bash")
        .stderr(Stdio::piped())
        .arg("-c")
        .arg("docker ps -q | xargs docker kill")
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to kill all docker containers: {e}"))?;

    print!("All docker containers successfully killed");

    Ok(())
}

pub async fn check_cluster_exists(instance_index: u32, cluster_name: String) -> Result<()> {
    let port: u32 = format!("{instance_index}8080").parse().unwrap();
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

    println!("Is cluster exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Cluster not exists")),
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

    println!("Is tenant exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Tenant not exists")),
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
        .arg(tenant)
        .spawn()?
        .wait_with_output()?
        .stdout;

    let is_exists = String::from_utf8(out)?
        .lines()
        .any(|line| line.contains(&namespace));

    println!("Is namespace exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Namespace not exists")),
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

