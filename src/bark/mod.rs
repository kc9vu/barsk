use crate::Resp;

mod commands;
pub(crate) mod de;
pub(crate) mod msg;

use anyhow::Result;
use curl::easy::Easy;

pub(crate) use commands::Level;

pub(crate) fn send_message(
    server: &str,
    device_key: &str,
    message: String,
    encrypted: bool,
) -> Result<Resp> {
    let mut handle = Easy::new();
    handle.useragent("curl/8.9.1")?;

    handle.url(&format!("{}/{}", server, device_key))?;
    handle.post(true)?;
    {
        let msg = if encrypted {
            &format!("ciphertext={}", urlencoding(&message))
        } else {
            &message
        };
        handle.post_fields_copy(msg.as_bytes())?;
    }

    let mut content = String::new();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            content.push_str(std::str::from_utf8(data).unwrap());
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(json5::from_str(&content)?)
}

fn urlencoding(s: &str) -> String {
    s.replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
        .replace(' ', "%20")
}
