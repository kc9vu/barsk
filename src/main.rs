use bark::{
    msg::{is_valid_cipher, Method},
    Level,
};
use clap::{ArgAction, ArgGroup, Parser};
use serde::{Deserialize, Serialize};

use crate::bark::send_message;

mod bark;

#[derive(Parser, Debug)]
#[command(
    name = "barsk",
    author,
    about,
    version,
    max_term_width = 80,
    group(
        ArgGroup::new("push level")
            .args(["active", "time_sensitive", "passive"])
            .conflicts_with("level")
    ),
)]
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
    #[arg(long, short, long_help = "Archiving the message to history. use --no-archive/-A to disable")]
    archive: bool,

    #[arg(long, short = 'A', overrides_with = "archive", hide = true)]
    no_archive: bool,

    /// The push interruption level
    #[arg(
        long,
        short,
        value_enum,
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

    /// The icon url will be shown in the notification bar
    #[arg(long)]
    icon: Option<String>,

    /// The badge number will be shown in the app icon
    #[arg(long, short)]
    badge: Option<String>,

    /// Push the encrypted message to server
    #[arg(long, short, long_help = "Push the encrypted message to server. Use --no-encrypt/-E to disable")]
    encrypt: bool,

    #[arg(long, short = 'E', overrides_with = "encrypt", hide = true)]
    no_encrypt: bool,

    /// Encryption method, e.g. aes128ecb aes256cbc
    #[arg(
        long,
        short,
        value_enum,
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
        let mut cli = <Self as Parser>::parse();
        if cli.active {
            cli.level = Some(Level::Active);
        } else if cli.time_sensitive {
            cli.level = Some(Level::TimeSensitive);
        } else if cli.passive {
            cli.level = Some(Level::Passive);
        }

        cli
    }

    fn to_msg<'a>(&'a self, conf: &'a Conf) -> Result<Msg<'a>, String> {
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
            level: self
                .level
                .as_ref()
                .or(conf.level.as_ref())
                .map(|l| l.into()),
            group: self.group.as_deref().or(conf.group.as_deref()),
            url: self.url.as_deref(),
            sound: self.sound.as_deref().or(conf.sound.as_deref()),
            icon: self.icon.as_deref().or(conf.icon.as_deref()),
            badge: self.badge.as_deref(),
        })
    }

    fn to_message(&self, conf: &Conf) -> Result<String, String> {
        let message = serde_json::to_string(&self.to_msg(conf)?).unwrap();

        if self.encrypt || conf.encrypt.unwrap_or(false) {
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
                return Err("Failed to load encryption config".into());
            };

            Ok(bark::msg::encrypt(&message, key, iv, method)?)
        } else {
            Ok(message)
        }
    }

    fn check_encryption(&self) -> Result<(), String> {
        if self.aes_key.is_none() {
            return Err("Missing aes_key".into());
        }
        is_valid_cipher(
            self.method,
            self.aes_key.as_ref().unwrap(),
            self.aes_iv.as_deref(),
        )?;

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct Conf {
    server: Option<String>,
    device_key: Option<String>,
    archive: Option<bool>,
    level: Option<Level>,
    group: Option<String>,
    icon: Option<String>,
    sound: Option<String>,
    encrypt: Option<bool>,
    method: Option<Method>,
    aes_key: Option<String>,
    aes_iv: Option<String>,
}

impl Conf {
    fn empty() -> Self {
        Self {
            server: None,
            device_key: None,
            archive: None,
            level: None,
            group: None,
            icon: None,
            sound: None,
            encrypt: None,
            method: None,
            aes_key: None,
            aes_iv: None,
        }
    }

    fn from_filepath(path: &str) -> Result<Self, String> {
        let reader = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };
        let config: Self = match serde_json::from_reader(reader) {
            Ok(conf) => conf,
            Err(e) => return Err(e.to_string()),
        };
        Ok(config)
    }

    fn from_executable_file() -> Result<Option<Self>, String> {
        let exe_path = match std::env::current_exe() {
            Ok(path) => path,
            Err(e) => return Err(e.to_string()),
        };
        let cur_dir = match exe_path.parent() {
            Some(dir) => dir,
            None => {
                return Err("Unable to locate the directory where the program is located".into())
            }
        };
        let config_file = cur_dir.join("bark.json");
        if config_file.exists() {
            Ok(Some(Self::from_filepath(config_file.to_str().unwrap())?))
        } else {
            Ok(None)
        }
    }

    fn check_encryption(&self) -> Result<(), String> {
        if self.method.is_none() {
            return Err("Miss encryption method in the config file".into());
        }
        if self.aes_key.is_none() {
            return Err("Miss aes_key in the config file".into());
        }
        is_valid_cipher(
            self.method.unwrap(),
            self.aes_key.as_ref().unwrap(),
            self.aes_iv.as_deref(),
        )?;
        Ok(())
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

    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<&'a str>,
}

#[allow(unused)]
#[derive(Deserialize)]
struct Res {
    code: u16,
    message: String,
    timestamp: u64,
}

fn run_command() -> Result<(), String> {
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
        // Send the message, print response message
        let res = send_message(
            server,
            cli.device_key
                .as_deref()
                .or(conf.device_key.as_deref())
                .unwrap(),
            message,
            cli.encrypt || conf.encrypt.unwrap_or(false),
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
        eprintln!("error: {error}");
    }
}
