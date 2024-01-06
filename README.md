# barsk

A bark cli written by Rust.

> Bark is a free, lightweight, push app for your iPhone with a simple interface call. [Official website](https://bark.day.app/#/)

## Features

- [x] Full functioning
- [x] Ecryption support(aes_256_cbc)
- [ ] More Encryption: aes_192_cbc, aes_128_cbc, aes_256_ecb, aes_192_ecb, aes_128_ecb
- [ ] Simplifying parameters: level

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
  -a, --archive               Archive the push. Flag can be overridden with --no-archive
  -l, --level <LEVEL>         Push interrupt level [possible values: active, timeSensitive, passive]
  -u, --url <URL>             URL on click
  -g, --group <GROUP>         Group the messages
      --badge <BADGE>         Push badge, can be any number
      --icon <ICON>           Setting custom icons
      --sound <SOUND>         Setting different ringtones
  -E, --encrypt               Encrypt message using AES, now only support aes_256_cbc. Flag can be overridden with --no-encrypt
      --key <KEY>             Used for encryption
      --iv <IV>               Used for encryption
  -F, --config <CONFIG_FILE>  Simplifying options with configuration files
  -z, --thats-all             Don't load default config
  -p, --dry-run               Print the message to be sent instead of sending it
  -S, --server <SERVER>       [http[s]://]host[:port]
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
    "key": "...",
    "iv": "..."
}
```
