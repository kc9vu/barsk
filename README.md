# barsk

version 0.5.0

A bark cli written by Rust.

> Bark is a free, lightweight, push app for your iPhone with a simple interface call. [Official website](https://bark.day.app/#/)

## Features

- [x] Full functioning
- [x] Ecryption support(aes_256_cbc)
- [x] More Encryption: aes_192_cbc, aes_128_cbc, aes_256_ecb, aes_192_ecb, aes_128_ecb
- [x] Simplifying parameters: level
- [ ] Docs

## Help

```plain
A bark cli written by Rust

Usage: barsk.exe [OPTIONS] <BODY>

Arguments:
  <BODY>  Push content

Options:
  -t, --title <TITLE>         Push title
  -C, --auto-copy             Automatically copy push content
  -c, --copy <COPY>           Copy the content at push, otherwise copy BODY
  -a, --archive               Archive the push. Can be overridden with --no-archive
  -l, --level <LEVEL>         Push interrupt level [possible values: active, timeSensitive/instant, passive]
                              Simple as --active, --time-sensitive/--instant, --passive
  -u, --url <URL>             URL on click
  -g, --group <GROUP>         Group the messages
      --badge <BADGE>         Push badge, can be any number
      --icon <ICON>           Setting custom icons
      --sound <SOUND>         Setting different ringtones
  -e, --encrypt               Encrypt message using AES. Can be overridden with --no-encrypt/-E
      --cipher <CIPHER>       Can be aes_xxx_cbc aes_xxx_ecb (xxx is 128, 192, 256)
      --key <KEY>             Used for encryption
      --iv <IV>               Used for encryption
  -F, --config <CONFIG_FILE>  Simplifying options with configuration files
  -z, --thats-all             Don't load default config
  -p, --dry-run               Print the message to be sent instead of sending it
  -s, --server <SERVER>       [http[s]://]host[:port]
  -d, --device <DEVICE_KEY>
  -h, --help                  Print help
  -V, --version               Print version
```

## Config file

```json
{
    "server": "https://api.day.app",
    "device_key": "...",
    "archive": false,
    "level": "active",
    "group": "From Windows",
    "icon": "https://www.example.com/favicon.ico",
    "sound": "bell",
    "encrypt": true,
    "cipher": "aes256cbc",
    "key": "...",
    "iv": "..."
}
```
