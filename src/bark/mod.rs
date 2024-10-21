pub mod de;
pub mod msg;

use curl::easy::{Easy, List};
use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize)]
pub struct Resp {
    pub code: u16,
    pub message: String,
    pub timestamp: u64,
}

pub fn send_message(
    server: &str,
    device_key: &str,
    message: String,
    encrypted: bool,
) -> Result<Resp, &'static str> {
    let mut handle = Easy::new();
    handle
        .useragent("curl/8.10.0")
        .or(Err("Failed to set user agent"))?;

    handle
        .url(&format!("{}/{}", server, device_key))
        .or(Err("Failed to set url"))?;
    handle.post(true).or(Err("Failed to set post method"))?;
    {
        let mut list = List::new();

        let msg = if encrypted {
            list.append("Content-Type: application/x-www-form-urlencoded")
                .or(Err("Failed to set Content-Type"))?;
            &format!("ciphertext={}", urlencoding(&message))
        } else {
            list.append("Content-Type: application/json; charset=utf-8")
                .or(Err("Failed to set Content-Type"))?;
            &message
        };

        handle.http_headers(list).or(Err("Failed to set headers"))?;
        handle
            .post_fields_copy(msg.as_bytes())
            .or(Err("Failed to set post fields"))?;
    }

    let mut content = String::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                content.push_str(std::str::from_utf8(data).unwrap());
                Ok(data.len())
            })
            .or(Err("Failed to set write function"))?;
        transfer.perform().or(Err("Failed to perform transfer"))?;
    }

    json5::from_str(&content).or(Err("Failed to parse response"))
}

fn urlencoding(s: &str) -> String {
    s.replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
        .replace(' ', "%20")
}
