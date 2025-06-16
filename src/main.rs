mod bark;
mod command;

use std::path::Path;

use anstream::{eprintln, println};
use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;
use reqwest::{
    Client,
    StatusCode,
    header::{self, HeaderMap, HeaderValue},
};
use serde::Deserialize;
use tokio::fs;

use crate::bark::Configuration;
use crate::command::Cli;

static API_SERVER: &str = "https://api.day.app";

#[allow(unused)]
#[derive(Deserialize, Debug)]
struct Resp {
    code: u16,
    message: String,
    timestamp: u64,
}

fn urlencoding(s: impl Into<String>) -> String {
    s.into()
        .replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
        .replace(' ', "%20")
        .replace('&', "%26")
}

fn hide_str(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    if s.len() < 10 {
        format!("{s}{}", &"xxxxxxxxxx"[..10 - s.len()])
    } else {
        format!("{}{}{}", &s[..4], "xxx", &s[s.len() - 6..])
    }
}

async fn read_config(path: &Path) -> Result<Configuration> {
    let content = fs::read_to_string(path).await?;
    let config = match path.extension().and_then(|s| s.to_str()) {
        Some("toml") => toml::from_str::<Configuration>(&content)?,
        _ => json5::from_str::<Configuration>(&content)?,
    };
    Ok(config)
}

async fn get_command() -> Result<Cli> {
    let mut cli = Cli::parse();
    #[cfg(debug_assertions)]
    println!("{:#?}", cli);

    if !cli.thats_all {
        if let Some(path) = cli.config_file() {
            let configuration = read_config(path).await?;
            #[cfg(debug_assertions)]
            println!("{:#?}", configuration);

            cli.service.merge(configuration.service);
            cli.push.update_storable(configuration.stored);
            cli.encryption
                .merge(configuration.encryption, cli.no_encrypt);
            cli.push.update_level(cli.level());
            cli.push.update_archive(cli.archive());
        }
    }

    Ok(cli)
}

async fn run_command() -> Result<()> {
    let cli = get_command().await?;
    #[cfg(debug_assertions)]
    println!("{:#?}", cli);

    let push = json5::to_string(&cli.push)?;

    let push = if cli.encryption.encrypted() {
        cli.encryption.encrypt(&push)?
    } else {
        push
    };

    let devices = cli.service.device_keys();
    if cli.dry_run {
        println!(
            "Will push to {}: {}",
            cli.service.server().cyan().italic(),
            devices
                .iter()
                .map(|dev| format!("{}", hide_str(dev).blue()))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!("{}", push.green());
    } else {
        if devices.is_empty() {
            return Ok(());
        }
        let client = Client::builder()
            .default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(header::USER_AGENT, HeaderValue::from_static("reqwest/0.12"));
                if cli.encryption.encrypted() {
                    headers.insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/x-www-form-urlencoded"),
                    );
                } else {
                    headers.insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/json; charset=utf-8"),
                    );
                }
                headers
            })
            .build()?;

        let body = if cli.encryption.encrypted() {
            format!("ciphertext={}", urlencoding(&push))
        } else {
            push
        };

        let mut handlers = Vec::with_capacity(devices.len());
        for dev in devices {
            let url = url::Url::parse(cli.service.server())?.join(dev)?;
            let client = client.clone();
            let body = body.clone();

            let handle = tokio::spawn(async move {
                let resp = client
                    .post(url)
                    .body(body)
                    .send()
                    .await?
                    .json::<Resp>()
                    .await?;

                #[cfg(debug_assertions)]
                println!("{:#?}", resp);

                let status = StatusCode::from_u16(resp.code)?;
                if status.is_success() {
                    println!("{}", resp.message.green());
                } else {
                    eprintln!("{}", resp.message.red());
                }
                Result::<_>::Ok(())
            });
            handlers.push(handle);
        }

        for handle in handlers {
            if let Err(er) = handle.await? {
                eprintln!("{}: {}", "error in sending message".red(), er);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(er) = run_command().await {
        eprintln!("{}: {}", "error".red(), er);
        std::process::exit(1);
    }
}
