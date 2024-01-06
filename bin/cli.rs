pub mod utils {
    pub mod secret {
        use openssl::base64::encode_block;
        use openssl::symm::{encrypt, Cipher};
        use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

        pub fn encrypt_message(key: &[u8], iv: &[u8], data: &[u8]) -> String {
            let cipher = Cipher::aes_256_cbc();
            let cipher_text = encrypt(cipher, key, Some(iv), data)
                .expect("Message encryption failed, check key and iv");
            let b64 = encode_block(&cipher_text);
            utf8_percent_encode(&b64, NON_ALPHANUMERIC).to_string()
        }
    }

    pub mod catch_all {
        pub fn with_default_protocol(address: &String) -> String {
            // maybe add condition address.contains("://") is better
            if !address[..10].contains("://") {
                format!("https://{address}")
            } else {
                address.to_owned()
            }
        }
    }
}

mod app {
    use std::fs::File;

    use clap::Parser;
    use serde::{Deserialize, Serialize};

    use super::utils::{secret::encrypt_message, catch_all::with_default_protocol};

    #[derive(Parser, Serialize, Clone)]
    #[command(name = "barsk", author, version, about, long_about = None)]
    pub struct Bark {
        #[arg(help = "Push content")]
        pub body: String,

        #[arg(short, long, help = "Push title")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,

        #[arg(short = 'C', long = "auto_copy", help = "Automatically copy push content")]
        #[serde(rename = "autoCopy")]
        pub auto_copy: bool,

        #[arg(short, long, help = "Copy the content at push, otherwise copy BODY")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub copy: Option<String>,

        #[arg(short = 'A', long = "archive", help = "Pass 1 to save the push, pass the others to not save the push")]
        #[serde(rename = "isArchive", skip_serializing_if = "Option::is_none")]
        pub is_archive: Option<String>,

        #[arg(
            short = 'L',
            long,
            help = "Push interrupt level [possible values: active, timeSensitive, passive]"
        )]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub level: Option<String>,

        #[arg(short = 'U', long, help = "URL on click")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,

        #[arg(short = 'G', long, help = "Group the messages")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub group: Option<String>,

        #[arg(long, help = "Push badge, can be any number")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub badge: Option<usize>,

        #[arg(long, help = "Setting custom icons")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon: Option<String>,

        #[arg(long, help = "Setting different ringtones")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub sound: Option<String>,

        #[arg(
            short = 'E',
            long,
            help = "Encrypt message using AES. Now only support aes_256_cbc"
        )]
        #[serde(skip_serializing)]
        pub encrypt: Option<bool>,

        #[arg(long, help = "Used for encryption")]
        #[serde(skip_serializing)]
        key: Option<String>,

        #[arg(long, help = "Used for encryption")]
        #[serde(skip_serializing)]
        iv: Option<String>,

        #[arg(short = 'F', long = "config", help = "Simplifying options with configuration files")]
        #[serde(skip_serializing)]
        config_file: Option<String>,

        #[arg(short = 'z', long, help = "Don't load default config")]
        #[serde(skip_serializing)]
        thats_all: bool,

        #[arg(long, help = "Print the message to be sent instead of sending it")]
        #[serde(skip_serializing)]
        dry_run: bool,

        #[arg(short = 'S', long, help = "[http[s]://]host[:port]")]
        #[serde(skip_serializing)]
        pub server: Option<String>,

        #[arg(short = 'd', long)]
        #[serde(skip_serializing)]
        pub device_key: Option<String>,
    }

    impl Bark {
        pub fn check(&self) {
            if self.server.is_none() || self.device_key.is_none() {
                panic!("Missing server or device_id")
            }
            if let Some(true) = self.encrypt {
                if self.key.is_none() || self.iv.is_none() {
                    panic!("When using encryption, key and iv must be provided at the same time")
                }
            }
        }

        pub fn dumps(&self) -> String {
            let data = serde_json::to_string(self).expect("Converting to JSON string");
            match self.encrypt {
                Some(true) => {
                    format!(
                        "ciphertext={}",
                        encrypt_message(
                            self.key.as_ref().unwrap().as_bytes(),
                            self.iv.as_ref().unwrap().as_bytes(),
                            data.as_bytes(),
                        )
                    )
                }
                _ => data,
            }
        }

        // pub fn read_config(&self) -> Option<Conf> {
        //     match self.config_file.as_ref() {
        //         Some(config_file) => {Some(Conf::from_file_yes(&config_file))},
        //         None => None,
        //     }
        // }

        /// Update missing options from config.
        pub fn update_with_config(&mut self, config: &Conf) {
            macro_rules! update_field {
                ($field:ident) => {
                    if self.$field.is_none() {
                        self.$field = config.$field.clone();
                    }
                };
            }
            update_field!(server);
            update_field!(device_key);
            update_field!(is_archive);
            update_field!(level);
            update_field!(group);
            update_field!(icon);
            update_field!(sound);
            update_field!(encrypt);

            if !self.thats_all && self.server.is_none() {
                self.server = Some(String::from("https://api.day.app"));
            }
            if self.key.is_some() ^ self.iv.is_some() {
                panic!("When using encryption, key and iv must be provided at the same time")
            } else if self.key.is_none() && self.iv.is_none() {
                self.key = config.key.clone();
                self.iv = config.iv.clone();
            }
        }

        /// Read missing arguments from the file. If no file is specified and
        /// _no_default != true_, automatically read from the default location.
        /// If the file is found, return a new Conf that supplements the missing
        /// options. Otherwise return a clone.
        pub fn by_file_config(&self) -> Self {
            let mut bark = self.clone();
            if let Some(config_file) = self.config_file.as_ref() {
                bark.update_with_config(&Conf::from_file(config_file));
            } else if !self.thats_all {
                if let Some(conf) = Conf::from_default_file() {
                    bark.update_with_config(&conf);
                }
            }

            bark
        }

        fn send(&self) {
            let client = reqwest::blocking::Client::new();
            let result = client
                .post(format!(
                    "{}/{}",
                    with_default_protocol(self.server.as_ref().unwrap()),
                    self.device_key.as_ref().unwrap(),
                ))
                .header(
                    "Content-Type",
                    match self.encrypt {
                        Some(true) => "application/x-www-form-urlencoded",
                        _ => "application/json; charset=utf-8",
                    },
                )
                .body(self.dumps())
                .send()
                .expect("Failed to send message! Please check network connection!")
                .json::<Res>()
                .expect("Unable to parse response format!");

            println!("{}", &result.message);
        }

        fn print(&self) {
            println!(
                "The message will be sent to {}/xxxxx",
                with_default_protocol(self.server.as_ref().unwrap()),
                // self.device_key.as_ref().unwrap(),
            );
            println!("{}", self.dumps());
        }

        pub fn execute(&self) {
            self.check();
            if self.dry_run {
                self.print();
            } else {
                self.send();
            }
        }
    }

    #[derive(Deserialize)]
    pub struct Conf {
        server: Option<String>,
        device_key: Option<String>,
        is_archive: Option<String>,
        level: Option<String>,
        group: Option<String>,
        icon: Option<String>,
        sound: Option<String>,
        encrypt: Option<bool>,
        key: Option<String>,
        iv: Option<String>,
    }

    impl Conf {
        fn check(&self) {
            if let Some(true) = self.encrypt {
                if self.key.is_some() ^ self.iv.is_some() {
                    panic!("The key and iv in the configuration file must exist at the same time")
                }
            }
        }

        fn from_file(path: &str) -> Self {
            let config: Self = serde_json::from_reader(File::open(path).expect("...")).expect("");
            config.check();
            config
        }

        fn from_default_file() -> Option<Self> {
            let config_file = std::env::current_exe()
                .expect("Cannot read current path")
                .parent()
                .expect("File path contains some characters that cannot be converted")
                .join("bark.json");
            if config_file.exists() {
                Some(Conf::from_file(config_file.to_str().expect(
                    "File path contains some characters that cannot be converted",
                )))
            } else {
                None
            }
        }
    }

    #[derive(Deserialize)]
    pub struct Res {
        // pub code: u16,
        pub message: String,
        // pub timestamp: u64,
    }
}

use app::Bark;
use clap::Parser;

fn main() {
    Bark::parse().by_file_config().execute();
}
