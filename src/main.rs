// Copyright (c) 2019-2024 Toradex AG
// SPDX-License-Identifier: MIT

use clap::Parser;
use serde::Deserialize;
use serde_yaml;
use shiplift::Docker;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize)]
struct Service {
    image: String,
    container_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ComposeFile {
    services: HashMap<String, Service>,
}

#[derive(serde::Serialize)]
struct ContainerInfo {
    id: String,
    image: String,
    names: Vec<String>,
    status: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    compose_file: String,

    #[arg(trailing_var_arg = true)]
    docker_args: Vec<String>,

    #[arg(long, default_value = "10")]
    monitor_duration: u64,
}

fn parse_compose_file(path: &str) -> Result<ComposeFile, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let compose: ComposeFile = serde_yaml::from_str(&content)?;
    Ok(compose)
}

fn run_docker_compose(compose_file: &str, args: &[&str]) -> std::io::Result<()> {
    println!("Running docker compose with args: {:?}", args);

    let mut cmd = Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(compose_file)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    cmd.wait()?;
    Ok(())
}

async fn monitor_container_statuses(
    container_names: Vec<String>,
    duration: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let docker = Docker::new();

    println!("Monitoring the following containers: {:?}", container_names);

    let mut container_info_list = Vec::new();

    for _ in 0..duration {
        let containers = docker.containers().list(&Default::default()).await?;

        for container in containers {
            if container
                .names
                .iter()
                .any(|name| container_names.contains(&name.trim_start_matches('/').to_string()))
            {
                let info = ContainerInfo {
                    id: container.id.clone(),
                    image: container.image.clone(),
                    names: container.names.clone(),
                    status: container.status.clone(),
                };

                container_info_list.push(info);
            }
        }

        sleep(Duration::from_secs(1)).await;
    }

    let json_output = serde_json::to_string_pretty(&container_info_list)?;
    println!("{}", json_output);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    println!("{:?}", cli);

    let compose = parse_compose_file(&cli.compose_file)?;
    println!("{:?}", compose);

    let container_names: Vec<String> = compose
        .services
        .values()
        .filter_map(|service| service.container_name.clone())
        .collect();
    println!("Container names to monitor: {:?}", container_names);

    let docker_args: Vec<&str> = cli.docker_args.iter().map(AsRef::as_ref).collect();
    println!("Docker args: {:?}", docker_args);

    run_docker_compose(&cli.compose_file, &docker_args)?;

    monitor_container_statuses(container_names, cli.monitor_duration).await?;

    Ok(())
}
