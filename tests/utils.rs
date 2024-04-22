use anyhow::Result;
use uuid::Uuid;
use assert_cmd::Command;

pub fn rand_instance_name() -> String {
    "pulsar-".to_string() + &Uuid::new_v4().to_string()
}

pub fn kill_all_docker_containers() {
    println!("Killing all docker containers");

    Command::new("bash")
        .arg("-c")
        .arg("docker ps -q | xargs docker kill")
        .assert()
        .success();

    print!("All docker containers successfully killed");
}

pub async fn check_cluster_exists(instance_index: u32, cluster_name: String) -> Result<()> {
    let port: u32 = format!("{instance_index}8080").parse().unwrap();
    let out = Command::new("pulsar-admin")
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("clusters")
        .arg("list")
        .assert()
        .get_output()
        .stdout
        .clone();

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
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("tenants")
        .arg("list")
        .assert()
        .get_output()
        .stdout
        .clone();

    let is_exists = String::from_utf8(out)?
        .lines()
        .any(|line| line.contains(&tenant));

    println!("Is tenant exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Cluster not exists")),
    }
}

pub async fn check_namespace_exists(cluster_index: u32, tenant: String, namespace: String) -> Result<()> {
    let port: u32 = format!("{cluster_index}8080").parse().unwrap();
    let out = Command::new("pulsar-admin")
        .arg("--admin-url")
        .arg(format!("http://localhost:{port}"))
        .arg("namespaces")
        .arg("list")
        .arg(tenant)
        .assert()
        .get_output()
        .stdout
        .clone();

    let is_exists = String::from_utf8(out)?
        .lines()
        .any(|line| line.contains(&namespace));

    println!("Is namespace exists: {is_exists}");

    match is_exists {
        true => Ok(()),
        false => Err(anyhow::anyhow!("Cluster not exists")),
    }
}

