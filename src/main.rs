mod docker_compose;
mod instance_config;

use anyhow::{anyhow, Error, Result};
use clap::{Parser, Subcommand};
use dirs::home_dir;
use docker_compose::docker_compose::{generate_instance, InstanceOutput, PrintInfo};
use instance_config::InstanceConfig;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct CreateCommandArgs {
    instance_name: String,

    #[clap(flatten)]
    pub instance_config: InstanceConfig,

    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct EditCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct LogsCommandArgs {
    pub instance_name: Option<String>,

    #[arg(short, long, default_value_t = false)]
    pub follow: bool,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct StartCommandArgs {
    pub instance_name: Option<String>,

    /// Follow container logs
    #[arg(long, default_value_t = false)]
    pub follow: bool,

    /// Keep containers running even if instance start failed
    #[arg(long, default_value_t = false)]
    pub no_kill: bool,

    /// Pull images before starting the instance
    #[arg(long, default_value_t = false)]
    pub pull: bool,

    /// Disable opening the browser after starting the instance
    #[arg(long, default_value_t = false)]
    pub no_open_browser: bool,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct PsCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct StopCommandArgs {
    pub instance_name: Option<String>,

    /// Stop all Pulsar instances
    #[arg(long, default_value_t = false)]
    pub all: bool,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct TemplateCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct DeleteCommandArgs {
    pub instance_name: String,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct PurgeCommandArgs {
    pub instance_name: String,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct LsCommandArgs {}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct StatsCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct DescribeCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct GetDefaultInstanceCommandArgs {}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct SetDefaultInstanceCommandArgs {
    instance_name: String,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct GetDefaultClusterCommandArgs {}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct SetDefaultClusterCommandArgs {
    cluster_index: u32,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct ExecCommandArgs {
    /// Pulsar instance name
    #[arg(long)]
    instance: Option<String>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    command: Vec<String>,

    /// Cluster index, e.g. 0, 1, 2
    #[arg(long)]
    cluster: Option<u32>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Pulsar instance
    #[command()]
    Create(CreateCommandArgs),

    /// Delete specified Pulsar instance
    #[command()]
    Delete(DeleteCommandArgs),

    /// Describe specified Pulsar instance
    #[command()]
    Describe(DescribeCommandArgs),

    /// Edit existing Pulsar instance
    #[command()]
    Edit(EditCommandArgs),

    /// Exec an arbitrary command, e.g. `puls exec pulsar-admin tenants list`
    #[command()]
    Exec(ExecCommandArgs),

    /// List all Pulsar instances
    #[command()]
    Ls(LsCommandArgs),

    /// Display logs for the specified Pulsar instance
    #[command()]
    Logs(LogsCommandArgs),

    /// List containers and services associated with the specified Pulsar instance
    #[command()]
    Ps(PsCommandArgs),

    /// Purge Pulsar instance data, but keep it's config
    #[command()]
    Purge(PurgeCommandArgs),

    /// Start Pulsar instance
    #[command()]
    Start(StartCommandArgs),

    /// Display resource usage statistics
    #[command()]
    Stats(StatsCommandArgs),

    /// Stop specified Pulsar instance
    #[command()]
    Stop(StopCommandArgs),

    /// Render docker-compose.yml template for the specified Pulsar instance
    #[command()]
    Template(TemplateCommandArgs),

    /// Get default Pulsar instance name
    #[command()]
    GetDefaultInstance(GetDefaultInstanceCommandArgs),

    /// Set default Pulsar instance name
    #[command()]
    SetDefaultInstance(SetDefaultInstanceCommandArgs),

    /// Get default cluster index. Clusters are sequentially are enumerated starting from 0
    #[command()]
    GetDefaultCluster(GetDefaultClusterCommandArgs),

    /// Set default default cluster index. Clusters are sequentially are enumerated starting from 0
    #[command()]
    SetDefaultCluster(SetDefaultClusterCommandArgs),
}

fn get_default_instance_name_file() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("default_instance_name"))
}

fn get_default_cluster_index_file() -> Result<PathBuf> {
    let config_dir = get_config_dir()?;
    Ok(config_dir.join("default_cluster_index"))
}

fn get_default_instance_name() -> Result<String> {
    let default_instance_name_file = get_default_instance_name_file()?;
    if !default_instance_name_file.exists() {
        println!("Default instance name isn't set. Setting it to \"default\"");
        create_cmd(CreateCommandArgs {
            instance_name: "default".to_string(),
            instance_config: InstanceConfig::default(),
            overwrite: false,
        })?;
        std::fs::write(default_instance_name_file.clone(), "default")?;
    }

    let default_instance_name = std::fs::read_to_string(default_instance_name_file.clone())?;
    Ok(default_instance_name.trim().to_string())
}

fn set_default_instance_name(instance_name: String) -> Result<()> {
    let default_instance_name_file = get_default_instance_name_file()?;
    std::fs::write(default_instance_name_file.clone(), instance_name)?;
    Ok(())
}

fn get_default_cluster_index() -> Result<u32> {
    let default_cluster_index_file = get_default_cluster_index_file()?;
    if !default_cluster_index_file.exists() {
        println!("Default cluster index isn't set. Setting it to 0");
        std::fs::write(default_cluster_index_file.clone(), "0")?;
    }

    let file_content = std::fs::read_to_string(default_cluster_index_file.clone())?;
    let default_cluster_index: u32 = file_content.trim().parse()?;
    Ok(default_cluster_index)
}

fn set_default_cluster_index(cluster_index: u32) -> Result<()> {
    let default_cluster_index_file = get_default_cluster_index_file()?;
    std::fs::write(
        default_cluster_index_file.clone(),
        cluster_index.to_string(),
    )?;
    Ok(())
}

fn exec_cmd(instance_name: String, cluster_index: u32, command: Vec<String>) -> Result<()> {
    let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

    // TODO - run in a separate container. Good enough for proof-of-concept.
    // To implement it, we should determine the docker network name first.
    let pulsar_proxy_service_name = format!("pulsar-proxy-cluster-{cluster_index}");

    let command_as_str = command.join(" ");
    let bash_args = [
        "bash",
        "-c",
        format!("export PATH=$PATH:/pulsar/bin; {command_as_str}").as_str(),
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect::<Vec<String>>();

    let mut docker_args: Vec<String> = [
        "compose",
        "-f",
        docker_compose_file.to_str().unwrap(),
        "exec",
        "--user",
        "pulsar",
        &pulsar_proxy_service_name,
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();

    docker_args.extend(bash_args);

    Command::new("docker").args(docker_args).spawn()?.wait()?;

    Ok(())
}

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = Path::new(&home_dir().unwrap()).join(".config").join("puls");
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir.clone())?;
    }
    Ok(config_dir)
}

fn get_instances_dir() -> Result<PathBuf> {
    let instances_dir = get_config_dir()?.join("instances");
    if !instances_dir.exists() {
        std::fs::create_dir_all(instances_dir.clone())?;
    }
    Ok(instances_dir)
}

fn list_instance_names() -> Result<Vec<String>> {
    let instances_dir = get_instances_dir()?;
    let instances = std::fs::read_dir(instances_dir)
        .expect("Failed to read instances directory")
        .map(|entry| {
            entry
                .expect("Failed to read instance directory")
                .file_name()
                .into_string()
                .expect("Failed to convert instance name to string")
        })
        .collect::<Vec<String>>();

    Ok(instances)
}

fn get_instance_dir(instance_name: String) -> Result<PathBuf> {
    let instances_dir = get_instances_dir()?;
    let instance_dir = instances_dir.join(instance_name);
    if !instance_dir.exists() {
        std::fs::create_dir_all(instance_dir.clone())?;
    }

    Ok(instance_dir)
}

fn is_instance_exists(instance_name: String) -> Result<bool> {
    let instances_dir = get_instances_dir()?;
    let instance_dir = instances_dir.join(instance_name);
    Ok(instance_dir.exists())
}

fn get_instance_config_file(instance_name: String) -> Result<PathBuf> {
    let instance_dir = get_instance_dir(instance_name)?;
    let instance_config_file = instance_dir.join("puls.yml");

    Ok(instance_config_file)
}

fn read_instance_config(instance_name: String) -> Result<InstanceConfig> {
    let instance_config_file = get_instance_config_file(instance_name)?;
    let instance_config_yaml =
        std::fs::read_to_string(instance_config_file.clone()).map_err(|open_err| {
            let err_msg = format!(
                "Failed to read instance config file at {}. {}",
                instance_config_file.display(),
                open_err
            );
            Error::msg(err_msg)
        })?;

    let instance_config = serde_yaml::from_str::<InstanceConfig>(&instance_config_yaml)?;
    Ok(instance_config)
}

fn get_instance_docker_compose_file(instance_name: String) -> Result<PathBuf> {
    let instance_dir = get_instance_dir(instance_name)?;
    let instance_docker_compose_file = instance_dir.join("docker-compose.yml");

    Ok(instance_docker_compose_file)
}

fn write_instance_config(
    instance_name: String,
    instance_config: InstanceConfig,
    is_overwrite: bool,
) -> Result<()> {
    let config_yaml = serde_yaml::to_string(&instance_config)?;

    let instance_name = instance_name.clone();
    let instance_name_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !instance_name_regex.is_match(&instance_name) {
        let err_msg = "Invalid instance name provided. Only alphanumeric characters, dashes, and underscores are allowed.".to_string();
        return Err(Error::msg(err_msg));
    }

    let is_already_exists = is_instance_exists(instance_name.clone())?;
    let instance_config_file = get_instance_config_file(instance_name.clone())?;

    if is_already_exists && !is_overwrite {
        let err_msg = format!(
            "Pulsar instance config with such name already exists at {}",
            instance_config_file.display()
        );
        return Err(Error::msg(err_msg));
    }

    std::fs::write(instance_config_file.clone(), config_yaml)?;

    println!(
        "Created a new Pulsar instance config file at {}",
        instance_config_file.display()
    );

    Ok(())
}

fn list_instance_configs(_args: LsCommandArgs) -> Result<Vec<InstanceConfig>> {
    let instances_dir = get_instances_dir()?;
    let instances = std::fs::read_dir(instances_dir)
        .expect("Failed to read instances directory")
        .map(|entry| {
            entry
                .expect("Failed to read instance directory")
                .file_name()
                .into_string()
                .expect("Failed to convert instance name to string")
        })
        .collect::<Vec<String>>();

    let mut instance_configs: Vec<InstanceConfig> = Vec::new();
    for instance in instances {
        let instance_config = read_instance_config(instance.clone());
        match instance_config {
            Ok(config) => {
                instance_configs.push(config);
            }
            Err(err) => {
                println!("Failed to read instance config for {}. {}", instance, err);
            }
        }
    }

    Ok(instance_configs)
}

fn create_cmd(args: CreateCommandArgs) -> Result<()> {
    println!("Creating a new Pulsar instance {}", args.instance_name);
    write_instance_config(args.instance_name, args.instance_config, args.overwrite)
}

fn template_cmd(args: TemplateCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);
    let instance_config = read_instance_config(instance_name.clone())?;
    let instance_output = generate_instance(instance_name, instance_config)?;
    println!("{}", instance_output.docker_compose_template);
    Ok(())
}

fn stats_cmd(args: StatsCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);

    let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

    let cmd_args = vec![
        "compose",
        "-f",
        docker_compose_file.to_str().unwrap(),
        "stats",
    ];

    Command::new("docker").args(cmd_args).spawn()?.wait()?;

    Ok(())
}

fn ls_cmd(args: LsCommandArgs) -> Result<()> {
    let instance_names = list_instance_names()?;

    for instance in instance_names {
        println!("{instance}");
    }

    Ok(())
}

fn edit_cmd(args: EditCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);

    let is_exists = is_instance_exists(instance_name.clone())?;
    if !is_exists {
        return Err(Error::msg(format!(
            "Instance \"{instance_name}\" does not exist"
        )));
    }

    let instance_config_file = get_instance_config_file(instance_name.clone())?
        .to_string_lossy()
        .to_string();

    println!(
        "Edit Pulsar instance {} config using default $EDITOR",
        instance_name
    );
    println!("Instance config file: {}", instance_config_file);

    let text_editor = std::env::var("EDITOR").unwrap_or("nano".to_string());
    Command::new(text_editor)
        .arg(instance_config_file)
        .spawn()?
        .wait()?;

    Ok(())
}

fn logs_cmd(args: LogsCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);
    let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

    let mut cmd_args = vec![
        "compose",
        "-f",
        docker_compose_file.to_str().unwrap(),
        "logs",
    ];

    if args.follow {
        cmd_args.push("--follow");
    }

    Command::new("docker").args(cmd_args).spawn()?.wait()?;

    Ok(())
}

fn describe_cmd(args: DescribeCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);
    let instance_config = read_instance_config(instance_name.clone())?;
    let instance_output = generate_instance(instance_name.clone(), instance_config)?;

    instance_output.print_info();

    Ok(())
}

fn ps_cmd(args: PsCommandArgs) -> Result<()> {
    fn ps_instance(instance_name: String) -> Result<()> {
        let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

        println!("Pulsar instance \"{}\"", instance_name);

        let cmd_args = vec!["compose", "-f", docker_compose_file.to_str().unwrap(), "ps"];

        Command::new("docker").args(cmd_args).spawn()?.wait()?;
        Ok(())
    }

    match args.instance_name {
        Some(name) => ps_instance(name),
        None => {
            let instance_names = list_instance_names()?;
            for instance_name in instance_names {
                ps_instance(instance_name).unwrap();
                println!();
            }
            Ok(())
        }
    }
}

fn start_cmd(args: StartCommandArgs) -> Result<InstanceOutput> {
    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);

    let is_exists = is_instance_exists(instance_name.clone())?;
    if !is_exists {
        println!("Pulsar instance with such name does not exist: {instance_name}");
        return Err(anyhow!("Run `puls create <instance_name>` first"));
    }

    let instance_config = read_instance_config(instance_name.clone())?;

    let instance_output = generate_instance(instance_name.clone(), instance_config)?;
    let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;
    std::fs::write(
        docker_compose_file.clone(),
        instance_output.docker_compose_template.clone(),
    )?;

    let instance_config = read_instance_config(instance_name.clone())?;
    println!(
        "Starting Pulsar instance \"{}\" with configuration:",
        instance_name
    );
    println!("\n{}", serde_yaml::to_string(&instance_config)?.trim());
    println!("---");

    let ctrlc_instance_name = instance_name.clone();
    ctrlc::set_handler(move || {
        println!(
            "Received process termination signal. Stopping Pulsar instance: {}",
            ctrlc_instance_name
        );

        let stop_cmd_result = stop_cmd(StopCommandArgs {
            instance_name: Some(ctrlc_instance_name.to_owned()),
            all: false,
        });

        match stop_cmd_result {
            Ok(_) => {}
            Err(err) => {
                println!(
                    "Failed to stop Pulsar instance: {}. {}",
                    ctrlc_instance_name, err
                );
                process::exit(1)
            }
        }
    })?;

    let mut docker_compose_args: Vec<&str> = vec![
        "compose",
        "-f",
        docker_compose_file.to_str().unwrap(),
        "up",
        "--remove-orphans",
    ];

    if args.pull {
        docker_compose_args.push("--pull");
        docker_compose_args.push("always");
    }

    if !args.follow {
        docker_compose_args.push("--wait");
        docker_compose_args.push("--detach");
    }

    let started_at = std::time::Instant::now();
    let exit_status = Command::new("docker")
        .args(docker_compose_args.clone())
        .spawn()?
        .wait()?;

    let completed_at = std::time::Instant::now();
    let seconds_elapsed = completed_at.duration_since(started_at).as_secs();
    let event_name = if exit_status.success() {
        "started"
    } else {
        "failed"
    };
    println!();
    println!("Pulsar instance \"{instance_name}\" {event_name} in {seconds_elapsed} seconds");

    if !exit_status.success() {
        if !args.no_kill {
            stop_cmd(StopCommandArgs {
                instance_name: Some(instance_name.clone()),
                all: false,
            })?;
        }

        println!();
        println!("- If you see that some container in the \"Error\" state, check the logs using `docker logs <container_name>`");
        println!("- You can check all containers logs using `puls logs {instance_name}`");
        println!("- Alternatively you can try to purge the instance data using `puls purge {instance_name}` and start it again by running `puls start {instance_name}`");
        println!();
        println!("If you think that something is wrong with puls, you can submit an issue here:");
        println!("https://github.com/tealtools/puls/issues");
        println!();
        let err_msg = "Have a good day!".to_string();
        return Err(Error::msg(err_msg));
    }

    Ok(instance_output)
}

fn stop_cmd(args: StopCommandArgs) -> Result<()> {
    fn stop_instance(instance_name: String) -> Result<()> {
        println!("Stopping Pulsar instance: {}", instance_name);
        let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

        Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(docker_compose_file.clone())
            .arg("rm")
            .arg("--stop")
            .arg("--force")
            .spawn()?
            .wait()?;

        Command::new("docker")
            .arg("compose")
            .arg("-f")
            .arg(docker_compose_file)
            .arg("down")
            .arg("--remove-orphans")
            .arg("--timeout")
            .arg("60")
            .spawn()?
            .wait()?;

        Ok(())
    }

    if args.all {
        let instance_names = list_instance_names()?;
        for instance_name in instance_names {
            stop_instance(instance_name).unwrap();
        }
        return Ok(());
    }

    let instance_name = args.instance_name.unwrap_or(get_default_instance_name()?);
    stop_instance(instance_name)?;

    Ok(())
}

fn delete_cmd(args: DeleteCommandArgs) -> Result<()> {
    let instance_name = args.instance_name;

    println!("Deleting Pulsar instance: {}", instance_name);

    purge_cmd(PurgeCommandArgs {
        instance_name: instance_name.clone(),
    })?;

    println!("Removing instance directory");
    let instance_dir = get_instance_dir(instance_name)?;
    std::fs::remove_dir_all(instance_dir).unwrap();

    Ok(())
}

fn purge_cmd(args: PurgeCommandArgs) -> Result<()> {
    let instance_name = args.instance_name;

    let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

    println!("Purging Pulsar instance data: {}", instance_name);

    println!("Running docker compose down");
    Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(docker_compose_file.clone())
        .arg("down")
        .arg("--remove-orphans")
        .arg("--volumes")
        .spawn()?
        .wait()?;

    println!("Running docker compose rm");
    Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(docker_compose_file.clone())
        .arg("rm")
        .arg("--force")
        .arg("--stop")
        .arg("--volumes")
        .spawn()?
        .wait()?;

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Template(args)) => template_cmd(args),
        Some(Commands::Create(args)) => {
            let event_name = if args.overwrite { "updated" } else { "created" };

            match create_cmd(args.clone()) {
                Ok(_) => {
                    println!("Pulsar instance successfully {event_name}");
                }
                Err(err) => {
                    let command_name = if args.overwrite { "update" } else { "create" };

                    println!("Failed to {command_name} Pulsar instance config");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Edit(args)) => {
            match edit_cmd(args.clone()) {
                Ok(_) => {
                    println!("Pulsar instance successfully updated");
                }
                Err(err) => {
                    println!("Failed to edit Pulsar instance config");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Purge(args)) => {
            let is_exists = is_instance_exists(args.instance_name.clone())?;
            if !is_exists {
                println!(
                    "Pulsar instance with such name does not exist: {}",
                    args.instance_name
                );
                process::exit(0)
            }

            match purge_cmd(args.clone()) {
                Ok(_) => {
                    println!(
                        "Pulsar instance data successfully purged: {}",
                        args.instance_name
                    );
                }
                Err(err) => {
                    println!(
                        "Failed to purge Pulsar instance data: {}",
                        args.instance_name
                    );
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Delete(args)) => {
            let is_exists = is_instance_exists(args.instance_name.clone())?;
            if !is_exists {
                println!(
                    "Pulsar instance with such name does not exist: {}",
                    args.instance_name
                );
                process::exit(0)
            }

            match delete_cmd(args.clone()) {
                Ok(_) => {
                    println!(
                        "Successfully deleted Pulsar instance: {}",
                        args.instance_name
                    );
                }
                Err(err) => {
                    println!("Failed to delete Pulsar instance: {}", args.instance_name);
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Describe(args)) => {
            match describe_cmd(args) {
                Ok(_) => {}
                Err(err) => {
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Ps(args)) => {
            match ps_cmd(args) {
                Ok(_) => {}
                Err(err) => {
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Start(args)) => {
            match start_cmd(args.clone()) {
                Ok(instance_output) => {
                    println!("Successfully started Pulsar instance");

                    instance_output.print_info();

                    if !args.no_open_browser {
                        for cluster_output in instance_output.clusters {
                            if let Some(url) = cluster_output.dekaf_host_url.clone() {
                                println!(
                                    "Opening Dekaf UI: {}",
                                    cluster_output
                                        .dekaf_host_url
                                        .clone()
                                        .unwrap_or("".to_string())
                                );
                                webbrowser::open(&url).unwrap();
                            }
                        }
                    }

                    println!(
                        "See the `puls describe` command to display instance information again"
                    );
                }
                Err(err) => {
                    println!("{}", err);
                    process::exit(1)
                }
            };

            Ok(())
        }
        Some(Commands::Stop(args)) => {
            match stop_cmd(args) {
                Ok(_) => {
                    println!("Successfully stopped Pulsar instance");
                }
                Err(err) => {
                    println!("Failed to stop Pulsar instance");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Ls(args)) => {
            match ls_cmd(args) {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to list Pulsar instances");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Stats(args)) => {
            match stats_cmd(args) {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed show stats");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Logs(args)) => {
            match logs_cmd(args) {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to display Pulsar instance logs");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::GetDefaultInstance(_)) => {
            let default_instance_name = get_default_instance_name()?;
            println!("{}", default_instance_name);
            Ok(())
        }
        Some(Commands::SetDefaultInstance(args)) => {
            set_default_instance_name(args.instance_name)?;
            Ok(())
        }
        Some(Commands::GetDefaultCluster(_)) => {
            let default_cluster_index = get_default_cluster_index()?;
            println!("{}", default_cluster_index);
            Ok(())
        }
        Some(Commands::SetDefaultCluster(args)) => {
            set_default_cluster_index(args.cluster_index)?;
            Ok(())
        }
        Some(Commands::Exec(args)) => {
            let instance_name = args.instance.unwrap_or(get_default_instance_name()?);
            let cluster_index = args.cluster.unwrap_or(get_default_cluster_index()?);

            exec_cmd(instance_name, cluster_index, args.command)?;

            Ok(())
        }
        None => {
            println!("No command provided");
            process::exit(1)
        }
    }
}
