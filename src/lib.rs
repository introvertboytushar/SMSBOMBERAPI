use worker::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

mod send_bombing;

type HmacSha256 = Hmac<Sha256>;

const TOKEN_EXPIRY_MS: u64 = 2 * 60 * 1000;

fn generate_token(secret: &str) -> String {
    let timestamp = Date::now().as_millis();
    let ts_str = timestamp.to_string();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC error");
    mac.update(ts_str.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    format!("{}.{}", ts_str, signature)
}

fn verify_token(token: &str, secret: &str) -> bool {
    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() != 2 { return false; }
    let ts: u64 = match parts[0].parse() { Ok(v) => v, Err(_) => return false };
    if Date::now().as_millis() - ts > TOKEN_EXPIRY_MS { return false; }
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC error");
    mac.update(parts[0].as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    parts[1] == expected
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // সব request send_bombing handler এ পাঠাও
    send_bombing::handle(req, &env).await
}
