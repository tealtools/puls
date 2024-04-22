mod docker_compose;

use anyhow::{Error, Result};
use clap::{Parser, Subcommand};
use dirs::config_local_dir;
use docker_compose::docker_compose::generate_template;
use rand::seq::IteratorRandom;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct RunCommandArgs {
    pub instance_name: Option<String>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct StopCommandArgs {
    pub instance_name: String,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct RenderCommandArgs {
    pub instance_name: String,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct DeleteCommandArgs {
    pub instance_name: String,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct LsCommandArgs {}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct DescribeCommandArgs {
    pub instance_name: String,
}

const DEFAULT_INSTANCE_NAME: &str = "default";
const DEFAULT_PULSAR_VERSION: &str = "3.2.2";
const DEFAULT_NUM_CLUSTERS: &str = "1";
const DEFAULT_NUM_BROKERS: &str = "1";
const DEFAULT_NUM_BOOKIES: &str = "1";
const DEFAULT_NUM_ZOOKEEPERS: &str = "1";

#[derive(Parser, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[command(version, about, long_about = None)]
pub struct InstanceConfig {
    #[arg(long, default_value = DEFAULT_INSTANCE_NAME)]
    pub name: String,

    #[arg(long, default_value = DEFAULT_PULSAR_VERSION)]
    pub pulsar_version: String,

    #[arg(long, default_value = DEFAULT_NUM_CLUSTERS)]
    pub num_clusters: u32,

    #[arg(long, default_value = DEFAULT_NUM_BROKERS)]
    pub num_brokers: u32,

    #[arg(long, default_value = DEFAULT_NUM_BOOKIES)]
    pub num_bookies: u32,

    #[arg(long, default_value = DEFAULT_NUM_ZOOKEEPERS)]
    pub num_zookeepers: u32,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        InstanceConfig {
            name: DEFAULT_INSTANCE_NAME.to_string(),
            pulsar_version: DEFAULT_PULSAR_VERSION.to_string(),
            num_clusters: DEFAULT_NUM_CLUSTERS.parse().unwrap(),
            num_brokers: DEFAULT_NUM_BROKERS.parse().unwrap(),
            num_bookies: DEFAULT_NUM_BOOKIES.parse().unwrap(),
            num_zookeepers: DEFAULT_NUM_ZOOKEEPERS.parse().unwrap(),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    #[command()]
    Render(RenderCommandArgs),

    #[command()]
    Create(InstanceConfig),

    #[command()]
    Delete(DeleteCommandArgs),

    #[command()]
    Ls(LsCommandArgs),

    #[command()]
    Run(RunCommandArgs),

    #[command()]
    Stop(StopCommandArgs),
}

struct PulsarCompose {}

impl PulsarCompose {
    fn get_config_dir(&self) -> Result<PathBuf> {
        let config_dir = Path::new(&config_local_dir().unwrap()).join("pulsar-compose");
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir.clone())?;
        }
        Ok(config_dir)
    }

    fn get_instances_dir(&self) -> Result<PathBuf> {
        let instances_dir = self.get_config_dir()?.join("instances");
        if !instances_dir.exists() {
            std::fs::create_dir_all(instances_dir.clone())?;
        }
        Ok(instances_dir)
    }

    fn get_instance_dir(&self, instance_name: String) -> Result<PathBuf> {
        let instances_dir = self.get_instances_dir()?;
        let instance_dir = instances_dir.join(instance_name);
        if !instance_dir.exists() {
            std::fs::create_dir_all(instance_dir.clone())?;
        }

        Ok(instance_dir)
    }

    fn is_instance_exists(&self, instance_name: String) -> Result<bool> {
        let instances_dir = self.get_instances_dir()?;
        let instance_dir = instances_dir.join(instance_name);
        Ok(instance_dir.exists())
    }

    fn get_instance_config_file(&self, instance_name: String) -> Result<PathBuf> {
        let instance_dir = self.get_instance_dir(instance_name)?;
        let instance_config_file = instance_dir.join("pulsar-compose.yml");

        Ok(instance_config_file)
    }

    fn read_instance_config(&self, instance_name: String) -> Result<InstanceConfig> {
        let instance_config_file = self.get_instance_config_file(instance_name)?;
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

    fn get_instance_docker_compose_file(&self, instance_name: String) -> Result<PathBuf> {
        let instance_dir = self.get_instance_dir(instance_name)?;
        let instance_docker_compose_file = instance_dir.join("docker-compose.yml");

        Ok(instance_docker_compose_file)
    }

    fn write_instance_config(&self, instance_config: InstanceConfig, is_overwrite: bool) -> Result<()> {
        let config_yaml = serde_yaml::to_string(&instance_config)?;

        let instance_name = instance_config.name.clone();
        let instance_name_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if !instance_name_regex.is_match(&instance_name) {
            let err_msg = "Invalid instance name provided. Only alphanumeric characters, dashes, and underscores are allowed.".to_string();
            return Err(Error::msg(err_msg));
        }

        let is_already_exists = self.is_instance_exists(instance_name.clone())?;
        let instance_config_file = self.get_instance_config_file(instance_name.clone())?;

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

    fn list_instances(&self, args: LsCommandArgs) -> Result<Vec<InstanceConfig>> {
        let instances_dir = self.get_instances_dir()?;
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
            let instance_config = self.read_instance_config(instance.clone());
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

    fn create_cmd(&self, instance_config: InstanceConfig, is_overwrite: bool) -> Result<()> {
        println!("Creating a new Pulsar instance {}", instance_config.name);
        self.write_instance_config(instance_config, is_overwrite)
    }

    fn render_cmd(&self, args: RenderCommandArgs) -> Result<()> {
        let instance_config = self.read_instance_config(args.instance_name.clone())?;
        let template = generate_template(instance_config);
        println!("{}", template);
        Ok(())
    }

    fn ls_cmd(&self, args: LsCommandArgs) -> Result<()> {
        let instances = self.list_instances(args)?;

        println!("Found {} Pulsar instances.", instances.len());
        println!();

        for instance in instances {
            println!("---");
            println!("{}", serde_yaml::to_string(&instance)?);
            println!();
        }

        Ok(())
    }

    fn run_cmd(&self, args: RunCommandArgs) -> Result<()> {
        let instance_name = match args.instance_name {
            Some(name) => name,
            None => {
                let default_instance_config = InstanceConfig {
                    ..Default::default()
                };

                println!("No instance name provided. Using default instance name: {}", default_instance_config.name);

                let is_already_exists = self.is_instance_exists(default_instance_config.name.clone())?;
                if !is_already_exists {
                    self.create_cmd(default_instance_config.clone(), false)?;
                }

                default_instance_config.name
            }
        };

        let instance_config = self.read_instance_config(instance_name.clone())?;

        let docker_compose_template = generate_template(instance_config);
        let docker_compose_file = self.get_instance_docker_compose_file(instance_name)?;
        std::fs::write(docker_compose_file.clone(), docker_compose_template)?;

        let mut docker_compose_up_cmd = Command::new("docker")
            // .stdin(Stdio::piped())
            // .stdout(Stdio::piped())
            // .stderr(Stdio::piped())
            .arg("compose")
            .arg("-f")
            .arg(docker_compose_file)
            .arg("up")
            .spawn()
            .expect("Failed to start docker compose");

        Ok(())
    }

    fn stop_cmd(&self, args: StopCommandArgs) -> Result<()> {
        let instance_name = args.instance_name;
        let instance_config_file = self.get_instance_config_file(instance_name)?;

        Ok(())
    }

    fn delete_cmd(&self, args: DeleteCommandArgs) -> Result<()> {
        let instance_name = args.instance_name;
        let instance_dir = self.get_instance_dir(instance_name)?;
        std::fs::remove_dir_all(instance_dir).unwrap();

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let pulsar_compose = PulsarCompose {};

    match args.command {
        Some(Commands::Render(args)) => pulsar_compose.render_cmd(args),
        Some(Commands::Create(instance_config)) => {
            match pulsar_compose.create_cmd(instance_config, false) {
                Ok(_) => {
                    println!("Successfully created a new Pulsar instance config");
                }
                Err(err) => {
                    println!("Failed to create a new Pulsar instance config");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Delete(args)) => {
            let is_exists = pulsar_compose.is_instance_exists(args.instance_name.clone())?;
            if !is_exists {
                println!("Nothing to delete. Pulsar instance with such name does not exist: {}", args.instance_name);
                process::exit(1)
            }

            match pulsar_compose.delete_cmd(args.clone()) {
                Ok(_) => {
                    println!("Successfully deleted Pulsar instance: {}", args.instance_name);
                }
                Err(err) => {
                    println!("Failed to delete Pulsar instance: {}", args.instance_name);
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Run(args)) => {
            match pulsar_compose.run_cmd(args) {
                Ok(_) => {
                    println!("Successfully started Pulsar instance");
                }
                Err(err) => {
                    println!("Failed to start Pulsar instance");
                    println!("{}", err);
                    process::exit(1)
                }
            };
            Ok(())
        }
        Some(Commands::Stop(args)) => {
            match pulsar_compose.stop_cmd(args) {
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
            match pulsar_compose.ls_cmd(args) {
                Ok(_) => {}
                Err(err) => {
                    println!("Failed to list Pulsar instances");
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
