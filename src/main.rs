mod bark;
mod command;

use command::Conf;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Parser;

fn read_config(path: &Path) -> Result<Conf> {
    Ok(json5::from_str(&fs::read_to_string(path)?)?)
}

fn get_command() -> Result<command::Cli> {
    let mut cli = command::Cli::parse();

    if let Some(config_file) = cli
        .config_file
        .as_ref()
        // .or(get_executable_config_path().as_ref())
    {
        let config = read_config(config_file)?;
        cli.conf = cli.conf.merge(config);
    }
    cli.conf.ensure_serialisation();

    Ok(cli)
}

fn run_command() -> Result<()> {
    let cli = get_command()?;

    let server = cli.conf.server.as_deref().unwrap_or("https://api.day.app");
    let message = cli.to_message()?;
    if cli.dry_run {
        // Print the message that would be sent
        println!(
            "Will send to {}/{}",
            server,
            cli.conf
                .device_key
                .as_deref()
                .map_or("no_device_key", |_| "xxxxxx")
        );
        println!("{}", message);
    } else {
        let device_key = cli
            .conf
            .device_key
            .as_deref()
            .ok_or(anyhow!("No device_key detect, please specify one"))?;
        // Send the message, print response message
        let res = bark::send_message(
            server,
            device_key,
            message,
            cli.conf.encryption.is_encrypted(),
        )
        .map_err(|e| anyhow!("{}", e))?;
        if res.code == 0 {
            println!("{}", res.message);
        } else {
            eprintln!("{}", res.message);
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run_command() {
        eprintln!("error: {}", error);
        std::process::exit(1);
    }
}
