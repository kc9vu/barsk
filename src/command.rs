use std::path::{Path, PathBuf};

use clap::{ArgAction, ArgGroup, Parser};

use crate::bark::{Encryption, Level, Push, Service};

#[derive(Parser, Debug)]
#[command(
    author = "kc9vu",
    version,
    about = "A cli tool to push notifications to bark servers",
    max_term_width = 80
)]
#[command(
    group = ArgGroup::new("push_level")
        .args(["critical", "active", "time_sensitive", "passive"])
        .conflicts_with("level")
        .required(false)
)]
pub struct Cli {
    // #[command(flatten)]
    // bark: Bark,
    #[command(flatten)]
    pub service: Service,

    /// The push to be sent
    #[command(flatten)]
    pub push: Push,

    /// Push of interruption level
    #[arg(long, hide = true)]
    critical: bool,

    /// Push of interruption level
    #[arg(long, hide = true)]
    active: bool,

    /// Push of interruption level
    #[arg(visible_alias = "instant", long, hide = true)]
    time_sensitive: bool,

    /// Push of interruption level
    #[arg(long, hide = true)]
    passive: bool,

    /// Tell the app to archive
    #[arg(overrides_with = "archive", long, short = 'a')]
    archive: bool,

    /// Tell the app not to archive
    #[arg(overrides_with = "archive", long, short = 'A')]
    no_archive: bool,

    /// Don't use action
    #[arg(long, overrides_with = "action", hide = true)]
    no_action: (),

    #[command(flatten)]
    pub encryption: Encryption,

    /// Don't encrypt, can be overrided
    #[arg(long, short = 'E', overrides_with = "encrypt", hide = true)]
    pub no_encrypt: bool,

    /// Path to configuration file that contains some popular options
    #[arg(env = "BARSK_CONFIG", long = "config", short = 'F', action = ArgAction::Set)]
    config_file: Option<PathBuf>,

    /// Don't load configuration from file
    #[arg(long, visible_alias = "no-file", short = 'z')]
    pub thats_all: bool,

    /// Just print push that will be sent, don't do sending
    #[arg(long, short = 'r')]
    pub dry_run: bool,
}

impl Cli {
    pub fn config_file(&self) -> Option<&Path> {
        self.config_file.as_deref()
    }

    pub fn level(&self) -> Option<Level> {
        if self.critical {
            Some(Level::Critical)
        } else if self.active {
            Some(Level::Active)
        } else if self.time_sensitive {
            Some(Level::TimeSensitive)
        } else if self.passive {
            Some(Level::Passive)
        } else {
            None
        }
    }

    pub fn archive(&self) -> Option<bool> {
        if self.archive {
            Some(true)
        } else if self.no_archive {
            Some(false)
        } else {
            None
        }
    }
}
