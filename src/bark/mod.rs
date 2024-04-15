use crate::Res;

mod commands;
// pub(crate) mod de;
pub(crate) mod msg;

pub(crate) use commands::Level;

pub(crate) fn send_message(
    server: &str,
    device_key: &str,
    message: String,
    encrypted: bool,
) -> Result<Res, String> {
    let client = reqwest::blocking::Client::new();
    match client
        .post(format!("{}/{}", server, device_key))
        .header(
            "Content-Type",
            if encrypted {
                "application/x-www-form-urlencoded"
            } else {
                "application/json; charset=utf-8"
            },
        )
        .body(if encrypted {
            format!("ciphertext={}", urlencoding(&message))
        } else {
            message
        })
        .send()
    {
        Ok(resp) => match resp.json::<Res>() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.to_string()),
        },
        Err(e) => Err(e.to_string()),
    }
}

fn urlencoding(s: &str) -> String {
    s.replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
        .replace(' ', "%20")
}
