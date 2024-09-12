# barsk

version 1.1.0

A bark cli written by Rust.

> Bark is a free, lightweight, push app for your iPhone with a simple interface call. [Official website](https://bark.day.app/#/)

## Features

- [x] Clear command line interface
- [x] Config file support
- [x] Encrypt message before push
- [x] Encryption support: aes_256_cbc, aes_192_cbc, aes_128_cbc, aes_256_ecb, aes_192_ecb, aes_128_ecb

## Help

```plain
Push to your iPhone with text, links and more!

Usage: barsk.exe [OPTIONS] <BODY>

Arguments:
  <BODY>
          Push content

Options:
  -t, --title <TITLE>
          The title will be shown on the notification bar

  -c, --copy <COPY>
          The content will be copied to clipboard or <BODY>

  -C, --auto-copy
          Auto copy the content to clipboard. Available under iOS 14.5

  -a, --archive
          Archiving the message to history. use --no-archive/-A to disable

  -l, --level <LEVEL>
          The push interruption level. Simple as --active, --time-sensitive (alias --instant), --passive

          [possible values: active, time-sensitive, instant, passive]

  -g, --group <GROUP>
          The group name in history messages

      --url <URL>
          The URL will be opened when the notification is clicked

      --sound <SOUND>
          The sound name or sound url will be played

  -r, --call
          Rings continuously for 30 seconds

      --icon <ICON>
          The icon url will be shown in the notification bar. Available above iOS 15

  -b, --badge <BADGE>
          The badge number will be shown in the app icon

  -e, --encrypt
          Push the encrypted message to server. Use --no-encrypt/-E to disable

  -m, --method <METHOD>
          Encryption method. Can be aes128cbc aes192cbc aes256cbc aes128ecb aes192ecb aes256ecb

          [default: aes128cbc]

      --aes-key <AES_KEY>
          The AES key for encryption

      --aes-iv <AES_IV>
          The AES initialization vector for encryption

  -s, --server <SERVER>
          The server url for push message

  -d, --device-key <DEVICE_KEY>
          The device key for push message

  -F, --config <CONFIG_FILE>
          The config file path

  -z, --thats-all
          Don't use any config file, it means run without adding any unspecified arguments

  -p, --dry-run
          Print message instead of sending it

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Config file

```json
{
    "server": "https://api.day.app",
    "device_key": "<KEY>",
    "encrypt": true,
    "method": "aes256cbc",
    "aes_key": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
    "aes_iv": "xxxxxxxxxxxxxxxx",
    "group": "Test",
    "sound": "birdsong"
}
```

## Crates Used

- [base64](https://github.com/marshallpierce/rust-base64)
- [clap](https://github.com/clap-rs/clap)
- [curl](https://github.com/alexcrichton/curl-rust)
- [json5](https://github.com/callum-oakley/json5-rs)
- [rust-crypto](https://github.com/DaGenix/rust-crypto)
- [serde](https://github.com/serde-rs/serde)

## LICENSE

[MIT](LICENSE)
