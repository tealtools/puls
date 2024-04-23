mod docker_compose;
mod instance_config;

use anyhow::{Error, Result};
use clap::{Parser, Subcommand};
use dirs::config_local_dir;
use docker_compose::docker_compose::generate_template;
use instance_config::InstanceConfig;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;

const DEFAULT_INSTANCE_NAME: &str = "default";

#[derive(Parser)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct CreateCommandArgs {
    #[clap(flatten)]
    pub instance_config: InstanceConfig,

    #[arg(long, default_value_t = false)]
    pub overwrite: bool,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct EditCommandArgs {
    pub instance_name: Option<String>
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

    #[arg(long, default_value_t = false)]
    pub debug: bool,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct PsCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct StopCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct RenderCommandArgs {
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
#[command(version, about, long_about = None, arg_required_else_help(true))]
pub struct DescribeCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Pulsar instance
    #[command()]
    Start(StartCommandArgs),

    /// Create a new Pulsar instance
    #[command()]
    Create(CreateCommandArgs),

    /// Edit existing Pulsar instance
    #[command()]
    Edit(EditCommandArgs),

    /// List all Pulsar instances
    #[command()]
    Ls(LsCommandArgs),

    /// Display logs for the specified Pulsar instance
    #[command()]
    Logs(LogsCommandArgs),

    /// List containers and services associated with the specified Pulsar instance
    #[command()]
    Ps(PsCommandArgs),

    /// Stop specified Pulsar instance
    #[command()]
    Stop(StopCommandArgs),

     /// Purge Pulsar instance data, but keep it's config
    #[command()]
    Purge(PurgeCommandArgs),

    /// Delete specified Pulsar instance
    #[command()]
    Delete(DeleteCommandArgs),

    /// Render docker-compose.yml template for the specified Pulsar instance
    #[command()]
    Render(RenderCommandArgs),
}

fn get_config_dir() -> Result<PathBuf> {
    let config_dir = Path::new(&config_local_dir().unwrap()).join("puls");
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

fn write_instance_config(instance_config: InstanceConfig, is_overwrite: bool) -> Result<()> {
    let config_yaml = serde_yaml::to_string(&instance_config)?;

    let instance_name = instance_config.name.clone();
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
    println!(
        "Creating a new Pulsar instance {}",
        args.instance_config.name
    );
    println!("S, overwrite {}", args.overwrite);
    write_instance_config(args.instance_config, args.overwrite)
}

fn render_cmd(args: RenderCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(DEFAULT_INSTANCE_NAME.to_string());
    let instance_config = read_instance_config(instance_name.clone())?;
    let template = generate_template(instance_config);
    println!("{}", template);
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
    let instance_name = args
        .instance_name
        .unwrap_or(DEFAULT_INSTANCE_NAME.to_string());

    let instance_config_file = get_instance_config_file(instance_name.clone())?.to_string_lossy().to_string();

    println!("Edit Pulsar instance {} config using default $EDITOR", instance_name);
    println!("Instance config file: {}", instance_config_file);

    let text_editor = std::env::var("EDITOR").unwrap_or("nano".to_string());
    Command::new(text_editor).arg(instance_config_file).spawn()?.wait()?;

    Ok(())
}

fn logs_cmd(args: LogsCommandArgs) -> Result<()> {
    let instance_name = args
        .instance_name
        .unwrap_or(DEFAULT_INSTANCE_NAME.to_string());
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

fn ps_cmd(args: PsCommandArgs) -> Result<()> {
    fn ps_instance(instance_name: String) -> Result<()> {
        let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;

        println!("Listing containers and services for Pulsar instance: {}", instance_name);
        println!("Docker Compose file: {}", docker_compose_file.display());

        let cmd_args = vec![
            "compose",
            "-f",
            docker_compose_file.to_str().unwrap(),
            "ps",
        ];

        Command::new("docker").args(cmd_args).spawn()?.wait()?;
        Ok(())
    }

    match args.instance_name {
        Some(name) => {
            ps_instance(name)
        }
        None => {
            let instance_names = list_instance_names()?;
            for instance_name in instance_names {
                ps_instance(instance_name).unwrap();
            }
            Ok(())
        }
    }
}

fn start_cmd(args: StartCommandArgs) -> Result<()> {
    let instance_name = match args.instance_name {
        Some(name) => name,
        None => {
            let default_instance_config = InstanceConfig {
                ..Default::default()
            };

            println!(
                "No instance name provided. Using default instance name: {}",
                default_instance_config.name
            );

            let is_already_exists = is_instance_exists(default_instance_config.name.clone())?;
            if !is_already_exists {
                create_cmd(CreateCommandArgs {
                    instance_config: default_instance_config.clone(),
                    overwrite: false,
                })?;
            }

            default_instance_config.name
        }
    };

    let instance_config = read_instance_config(instance_name.clone())?;

    let docker_compose_template = generate_template(instance_config);
    let docker_compose_file = get_instance_docker_compose_file(instance_name.clone())?;
    std::fs::write(docker_compose_file.clone(), docker_compose_template)?;

    let instance_config = read_instance_config(instance_name.clone())?;
    println!("Starting Pulsar instance: {}", instance_name);
    println!(
        "Instance configuration:\n---\n{}",
        serde_yaml::to_string(&instance_config)?.trim()
    );
    println!("---");

    let ctrlc_instance_name = instance_name.clone();
    ctrlc::set_handler(move || {
        println!(
            "Received process termination signal. Stopping Pulsar instance: {}",
            ctrlc_instance_name
        );

        let stop_cmd_result = stop_cmd(StopCommandArgs {
            instance_name: Some(ctrlc_instance_name.to_owned()),
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

    println!("Is debug mode: {}", args.debug);

    if !args.debug {
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
    println!("Pulsar instance started in {seconds_elapsed} seconds");

    if !exit_status.success() {
        let err_msg = format!("Failed to start Pulsar instance: {}", instance_name);
        return Err(Error::msg(err_msg));
    }

    Ok(())
}

fn stop_cmd(args: StopCommandArgs) -> Result<()> {
    let instance_name = args.instance_name.unwrap_or(DEFAULT_INSTANCE_NAME.to_string());

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
        Some(Commands::Render(args)) => render_cmd(args),
        Some(Commands::Create(args)) => {
            let event_name = if args.overwrite {
                "updated"
            } else {
                "created"
            };

            match create_cmd(args.clone()) {
                Ok(_) => {
                    println!("Pulsar instance successfully {event_name}");
                }
                Err(err) => {
                    let command_name = if args.overwrite {
                        "update"
                    } else {
                        "create"
                    };

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
                    println!("Failed to purge Pulsar instance data: {}", args.instance_name);
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
            match start_cmd(args) {
                Ok(_) => {
                    println!("Successfully started Pulsar instance");
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
        None => {
            println!("No command provided");
            process::exit(1)
        }
    }
}
