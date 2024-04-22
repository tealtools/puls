use anyhow::Result;
use assert_cmd::Command;
use uuid::Uuid;

pub fn rand_instance_name() -> String {
    "pulsar-".to_string() + &Uuid::new_v4().to_string()
}

async fn check_cluster_exists(instance_index: u32, cluster_name: String) -> Result<()> {
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

#[tokio::test(flavor = "multi_thread")]
async fn test_run_single_cluster_x1() {
    let instance_name = rand_instance_name();

    let instance_name_clone = instance_name.clone();
    tokio::spawn(async move {
        println!("Creating instance: {instance_name_clone}");
        Command::cargo_bin("pulsar-compose")
            .unwrap()
            .arg("create")
            .arg("--name")
            .arg(instance_name_clone.clone())
            .assert()
            .success();

        println!("Running instance: {instance_name_clone}");
        Command::cargo_bin("pulsar-compose")
            .unwrap()
            .arg("run")
            .arg(instance_name_clone.clone())
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
    }).await.unwrap();

    assert!(is_cluster_exists);
}
