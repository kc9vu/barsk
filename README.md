# barsk

A cli tool to push notifications to bark servers written by Rust.

> Bark is a free, lightweight, push app for your iPhone with a simple interface call. [Official website](https://bark.day.app/#/)

## Help

```plain
A cli tool to push notifications to bark servers

Usage: barsk.exe [OPTIONS] --body <BODY>

Options:
  -s, --server <SERVER>
          The server address of bark api service, default is https://api.day.app
  -d, --device-key <DEVICE_KEY>
          Device key to receive push
  -D, --device-keys <DEVICE_KEYS>
          A list of device key to receive push
  -t, --title <TITLE>
          Push title
  -T, --subtitle <SUBTITLE>
          Push subtitle
  -b, --body <BODY>
          Push content
  -l, --level <LEVEL>
          Push interrupt level [possible values: critical, active, time-sensitive, passive]
  -v, --volume <VOLUME>
          Important warning notification volume
  -B, --badge <BADGE>
          Push angle marker, can be any number
  -R, --call
          Repeat notification ringtone
  -C, --auto-copy
          Automatically copy push content
  -c, --copy <COPY>
          Specify the copied content. If you do not pass this parameter, the entire push content will be copied
  -u, --url <URL>
          The URL that jumps when clicking push
      --action
          When "none" is transmitted, clicking push will not pop up
  -S, --sound <SOUND>
          Set different ringtones
  -I, --icon <ICON>
          Set custom icons
  -g, --group <GROUP>
          Group messages
  -a, --archive
          Tell the app to archive
  -A, --no-archive
          Tell the app not to archive
  -e, --encrypt
          Send encrypted push. Make sure not to use encryption, use --no-encryption, simple as -E
  -m, --modes <MODES>
          Encrypt modes [default: aes256cbc]
      --aes-key <KEY>
          For encryption [aliases: --aeskey]
      --aes-iv <IV>
          For encryption [aliases: --aesiv]
  -F, --config <CONFIG_FILE>
          Path to configuration file that contains some popular options [env: BARSK_CONFIG=]
  -z, --thats-all
          Don't load configuration from file [aliases: --no-file]
  -r, --dry-run
          Just print push that will be sent, don't do sending
  -h, --help
          Print help
  -V, --version
          Print version
```

## Config file

```json
{
    "server": "https://api.day.app",
    "device_key": "token0",
    "device_keys": ["token1", "token2"],

    "encrypt": false,
    "modes": "aes256cbc",
    "aes_key": "0123456789abcdef0123456789abcdef",
    "aes_iv": "0123456789abcdef",

    "sound": "birdsong",
    "icon": "https://bark.day.app/_media/Icon.png",
    "group": "Normal",
    "archive": null,  // true / false
}
```

```toml
server = "https://api.day.app"
device_key = "token0"
device_keys = ["token1", "token2"]

encrypt = false
modes = "aes256cbc"
aes_key = "0123456789abcdef0123456789abcdef"
aes_iv = "0123456789abcdef"

sound = "birdsong"
icon = "https://bark.day.app/_media/Icon.png"
group = "Normal"
archive = None # true / false

```
## Core crates used

- [aes](https://github.com/RustCrypto/block-ciphers)
- [cbc](https://github.com/RustCrypto/block-modes)
- [clap](https://github.com/clap-rs/clap)
- [reqwest](https://github.com/seanmonstar/reqwest)

## LICENSE

[MIT](LICENSE)
