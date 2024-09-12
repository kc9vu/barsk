use anyhow::{bail, Result};
use bark::{
    msg::{is_valid_cipher, Method},
    Level,
};
use clap::{ArgAction, ArgGroup, Parser};
use serde::{Deserialize, Serialize};

mod bark;

#[derive(Parser, Debug)]
#[command(
    bin_name = "barsk",
    author = "kc9vu",
    about = "Push to your iPhone with text, links and more!",
    version,
    max_term_width = 80
)]
#[command(
    group(
    ArgGroup::new("push level")
        .args(["active", "time_sensitive", "passive"])
        .conflicts_with("level")
))]
struct Cli {
    /// Push content
    body: String,

    /// The title will be shown on the notification bar
    #[arg(long, short)]
    title: Option<String>,

    /// The content will be copied to clipboard or <BODY>
    #[arg(long, short)]
    copy: Option<String>,

    /// Auto copy the content to clipboard. Available under iOS 14.5
    #[arg(long, short = 'C')]
    auto_copy: bool,

    /// Archiving the message to history
    #[arg(
        long,
        short,
        long_help = "Archiving the message to history. use --no-archive/-A to disable"
    )]
    archive: bool,

    #[arg(long, short = 'A', overrides_with = "archive", hide = true)]
    no_archive: bool,

    /// The push interruption level
    #[arg(
        long,
        short,
        // value_enum,
        // hide_possible_values = true,
        ignore_case = true,
        long_help = "The push interruption level. Simple as --active, --time-sensitive (alias --instant), --passive"
    )]
    level: Option<Level>,

    #[arg(long, hide = true)]
    active: bool,

    #[arg(long, visible_alias = "instant", hide = true)]
    time_sensitive: bool,

    #[arg(long, hide = true)]
    passive: bool,

    /// The group name in history messages
    #[arg(long, short)]
    group: Option<String>,

    /// The URL will be opened when the notification is clicked
    #[arg(long)]
    url: Option<String>,

    /// The sound name or sound url will be played
    #[arg(long)]
    sound: Option<String>,

    /// Rings continuously for 30 seconds
    #[arg(alias = "call", long, short = 'r')]
    persistent_ringing: bool,

    /// The icon url will be shown in the notification bar. Available above iOS 15
    #[arg(long)]
    icon: Option<String>,

    /// The badge number will be shown in the app icon
    #[arg(long, short)]
    badge: Option<String>,

    /// Push the encrypted message to server
    #[arg(
        long,
        short,
        long_help = "Push the encrypted message to server. Use --no-encrypt/-E to disable"
    )]
    encrypt: bool,

    #[arg(long, short = 'E', overrides_with = "encrypt", hide = true)]
    no_encrypt: bool,

    /// Encryption method, e.g. aes128ecb aes256cbc
    #[arg(
        long,
        short,
        // value_enum,
        default_value = "aes128cbc",
        hide_possible_values = true,
        ignore_case = true,
        long_help = "Encryption method. Can be aes128cbc aes192cbc aes256cbc aes128ecb aes192ecb aes256ecb"
    )]
    method: Method,

    /// The AES key for encryption
    #[arg(long)]
    aes_key: Option<String>,

    /// The AES initialization vector for encryption
    #[arg(long)]
    aes_iv: Option<String>,

    /// The server url for push message
    #[arg(long, short)]
    server: Option<String>,

    /// The device key for push message
    #[arg(long, short)]
    device_key: Option<String>,

    /// The config file path
    // Use ArgAction::Set can use multiple config_file, the last one will be used
    #[arg(long = "config", short = 'F', action = ArgAction::Set)]
    config_file: Option<String>,

    /// Don't use any config file, it means run without adding any unspecified arguments
    #[arg(long, short = 'z')]
    thats_all: bool,

    /// Print message instead of sending it
    #[arg(long, short = 'p')]
    dry_run: bool,
}

impl Cli {
    fn parse() -> Self {
        let mut cli: Cli = Parser::parse();
        if cli.active {
            cli.level = Some(Level::Active);
        } else if cli.time_sensitive {
            cli.level = Some(Level::TimeSensitive);
        } else if cli.passive {
            cli.level = Some(Level::Passive);
        }

        cli
    }

    fn to_msg<'a>(&'a self, conf: &'a Conf) -> Result<Msg<'a>> {
        Ok(Msg {
            body: self.body.as_str(),
            title: self.title.as_deref(),
            copy: self.copy.as_deref(),
            auto_copy: if self.auto_copy { Some(()) } else { None },
            archive: if self.no_archive {
                Some("0")
            } else if self.archive {
                Some("1")
            } else {
                conf.archive.map(|a| if a { "1" } else { "0" })
            },
            level: self.level.as_ref().or(conf.level.as_ref()),
            group: self.group.as_deref().or(conf.group.as_deref()),
            url: self.url.as_deref(),
            sound: self.sound.as_deref().or(conf.sound.as_deref()),
            call: self.persistent_ringing,
            badge: self.badge.as_deref(),
        })
    }

    fn to_message(&self, conf: &Conf) -> Result<String> {
        let message = json5::to_string(&self.to_msg(conf)?)?;

        if !self.no_encrypt && (self.encrypt || conf.encrypt.unwrap_or(false)) {
            let (key, iv, method) = if self.check_encryption().is_ok() {
                // Use encryption config from command line
                (
                    self.aes_key.as_deref().unwrap(),
                    self.aes_iv.as_deref(),
                    self.method,
                )
            } else if conf.check_encryption().is_ok() {
                // Use encryption config from config file
                (
                    conf.aes_key.as_deref().unwrap(),
                    conf.aes_iv.as_deref(),
                    conf.method.unwrap(),
                )
            } else {
                bail!("Failed to load encryption config");
            };

            bark::msg::encrypt(&message, key, iv, method)
        } else {
            Ok(message)
        }
    }

    fn check_encryption(&self) -> Result<()> {
        if self.aes_key.is_none() {
            bail!("Missing aes_key");
        }
        is_valid_cipher(
            self.method,
            self.aes_key.as_ref().unwrap(),
            self.aes_iv.as_deref(),
        )?;

        Ok(())
    }
}

#[derive(Deserialize, Default, Debug)]
struct Conf {
    server: Option<String>,
    device_key: Option<String>,
    archive: Option<bool>,
    level: Option<Level>,
    group: Option<String>,
    // icon: Option<String>,
    sound: Option<String>,
    encrypt: Option<bool>,
    method: Option<Method>,
    aes_key: Option<String>,
    aes_iv: Option<String>,
}

impl Conf {
    fn empty() -> Self {
        Default::default()
    }

    fn from_filepath(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(json5::from_str(&content)?)
    }

    fn from_executable_file() -> Result<Option<Self>> {
        let exe_path = std::env::current_exe()?;
        let cur_dir = match exe_path.parent() {
            Some(dir) => dir,
            None => {
                return Err(anyhow::anyhow!(
                    "Unable to locate the directory where the program is located"
                ))
            }
        };
        let config_file = cur_dir.join("bark.json");
        if config_file.exists() {
            Ok(Some(Self::from_filepath(config_file.to_str().unwrap())?))
        } else {
            Ok(None)
        }
    }

    fn check_encryption(&self) -> Result<()> {
        if self.method.is_none() {
            return Err(anyhow::anyhow!("Miss encryption method in the config file"));
        }
        if self.aes_key.is_none() {
            return Err(anyhow::anyhow!("Miss aes_key in the config file"));
        }
        is_valid_cipher(
            self.method.unwrap(),
            self.aes_key.as_ref().unwrap(),
            self.aes_iv.as_deref(),
        )
    }
}

#[derive(Serialize, Debug)]
struct Msg<'a> {
    body: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    copy: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    auto_copy: Option<()>,

    #[serde(rename = "isArchive", skip_serializing_if = "Option::is_none")]
    archive: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "bark::de::serialize_level")]
    level: Option<&'a Level>,

    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<&'a str>,

    #[serde(skip_serializing_if = "bark::de::is_false", serialize_with = "bark::de::serialize_call")]
    call: bool,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // icon: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<&'a str>,
}

#[allow(unused)]
#[derive(Deserialize)]
struct Resp {
    code: u16,
    message: String,
    timestamp: u64,
}

fn run_command() -> Result<()> {
    let cli = Cli::parse();

    // Load a config from --config-file or from default path or empty
    let conf = if cli.thats_all {
        Conf::empty()
    } else if cli.config_file.is_some() {
        Conf::from_filepath(cli.config_file.as_ref().unwrap())?
    } else if let Some(config) = Conf::from_executable_file()? {
        config
    } else {
        Conf::empty()
    };

    let server = cli
        .server
        .as_deref()
        .or(conf.server.as_deref())
        .unwrap_or("https://api.day.app");
    let message = cli.to_message(&conf)?;

    if cli.dry_run {
        // Print the message that would be sent
        println!(
            "Will send to {}/{}",
            server,
            cli.device_key
                .as_deref()
                .or(conf.device_key.as_deref())
                .map(|_| "xxxxxx")
                .unwrap_or("no_device_key")
        );
        println!("{}", message);
    } else {
        let device_key = match cli.device_key.as_deref().or(conf.device_key.as_deref()) {
            Some(value) => value,
            None => bail!("No device_key detect, please specify one"),
        };

        // Send the message, print response message
        let res = bark::send_message(
            server,
            device_key,
            message,
            !cli.no_encrypt && (cli.encrypt || conf.encrypt.unwrap_or(false)),
        )?;
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
    }
}
