mod config;
mod docker_compose;

use clap::{Parser, Subcommand};
use config::config::PulsarInstanceConfig;
use dirs::config_local_dir;
use docker_compose::docker_compose::generate_template;
use std::path::Path;
use std::process::{exit, Command};
use regex::Regex;
use std::process;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Clone, Debug)]
#[command(version, about, long_about = None)]
pub struct UpCommandArgs {
    pub instance_name: String,
}

#[derive(Subcommand)]
enum Commands {
    #[command()]
    Render(PulsarInstanceConfig),

    #[command()]
    Create(PulsarInstanceConfig),

    #[command()]
    Up(UpCommandArgs),
}

fn main() {
    let args = Cli::parse();

    let config_dir = Path::new(&config_local_dir().unwrap()).join("pulsar-compose");
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir.clone()).expect("Failed to create config directory");
    }

    let instances_dir = config_dir.join("instances");
    if !instances_dir.exists() {
        std::fs::create_dir_all(instances_dir.clone())
            .expect("Failed to create instances directory");
    }

    match args.command {
        Some(Commands::Render(instance_config)) => {
            let template = generate_template(instance_config);
            println!("{}", template)
        }
        Some(Commands::Create(instance_config)) => {
            let config_yaml = serde_yaml::to_string(&instance_config)
                .expect("Failed to serialize Pulsar instance config");

            let instance_name = instance_config.name.clone();
            let instance_name_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
            if !instance_name_regex.is_match(&instance_name) {
                println!("Invalid instance name provided. Only alphanumeric characters, dashes, and underscores are allowed.");
                process::exit(1);
            }

            let is_already_exists = instances_dir.join(instance_name.clone()).exists();

            if is_already_exists {
                println!(
                    "Pulsar instance config with such name already exists at {}",
                    instances_dir.join(instance_name).display()
                );
                process::exit(1);
            }

            std::fs::create_dir(instances_dir.join(instance_name.clone()))
                .expect("Failed to create a new Pulsar instance config directory");

            let config_file = instances_dir
                .join(instance_config.name)
                .join(format!("{}.yml", instance_name.clone()));

            std::fs::write(config_file.clone(), config_yaml)
                .expect("Failed to write Pulsar instance config");
            println!(
                "Created a new Pulsar instance config file at {}",
                config_file.display()
            )
        }
        Some(Commands::Up(args)) => {
            let instance_name = args.instance_name.clone();

            let instance_config_file = instances_dir
                .join(instance_name.clone())
                .join(format!("{}.yml", instance_name.clone()));
            let instance_config_yaml = std::fs::read_to_string(instance_config_file)
                .expect("Failed to read Pulsar instance config file");

            let instance_config =
                serde_yaml::from_str::<PulsarInstanceConfig>(&instance_config_yaml)
                    .expect("Failed to parse Pulsar instance config");

            let template = generate_template(instance_config);

            let template_out_file = config_dir.join("docker-compose.yml");

            std::fs::write(template_out_file.clone(), template)
                .expect("Failed to write docker compose template");

            let mut cmd = Command::new("docker")
                .arg("compose")
                .arg("-f")
                .arg(template_out_file)
                .arg("up")
                .spawn()
                .expect("Failed to start docker compose");
        }
        None => {
            println!("No command provided")
        }
    }
}
