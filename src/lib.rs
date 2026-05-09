use worker::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

// ==================== ALLOWED ORIGINS ====================
const ALLOWED_ORIGINS: &[&str] = &[
    "https://sms-bomber-it.vercel.app",
    "https://customsms-it.vercel.app",
    "https://smsbomber.introvertboytushar.workers.dev",
];

// Token কতক্ষণ valid — 2 মিনিট (milliseconds)
const TOKEN_EXPIRY_MS: u64 = 2 * 60 * 1000;

// ==================== HELPERS ====================

fn get_origin(req: &Request) -> String {
    req.headers()
        .get("Origin").unwrap_or(None)
        .or_else(|| req.headers().get("Referer").unwrap_or(None))
        .unwrap_or_default()
}

fn is_origin_allowed(origin: &str) -> bool {
    if origin.is_empty() {
        return false;
    }
    ALLOWED_ORIGINS.iter().any(|o| origin.starts_with(o))
}

fn cors_headers(origin: &str) -> Headers {
    let mut headers = Headers::new();
    if is_origin_allowed(origin) {
        headers.set("Access-Control-Allow-Origin", origin).unwrap();
    } else {
        headers.set("Access-Control-Allow-Origin", "null").unwrap();
    }
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token, X-User-ID").unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    headers
}

fn generate_token(secret: &str) -> String {
    let timestamp = Date::now().as_millis();
    let ts_str = timestamp.to_string();

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC init failed");
    mac.update(ts_str.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    format!("{}.{}", ts_str, signature)
}

fn verify_token(token: &str, secret: &str) -> bool {
    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() != 2 {
        return false;
    }

    let timestamp_str = parts[0];
    let signature = parts[1];

    // Timestamp parse করো
    let ts: u64 = match timestamp_str.parse() {
        Ok(v) => v,
        Err(_) => return false,
    };

    // Token expired কিনা চেক করো
    let now = Date::now().as_millis();
    if now - ts > TOKEN_EXPIRY_MS {
        return false;
    }

    // Signature verify করো
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC init failed");
    mac.update(timestamp_str.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());

    signature == expected
}

// ==================== MAIN ====================

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        // ── OPTIONS: CORS preflight ──
        .options("/*", |req, _ctx| {
            let origin = get_origin(&req);
            let headers = cors_headers(&origin);
            Ok(Response::empty()?.with_headers(headers))
        })

        // ── GET /api/token: Token দাও ──
        .get_async("/token", |req, ctx| async move {
            let origin = get_origin(&req);
            let headers = cors_headers(&origin);

            if !is_origin_allowed(&origin) {
                return Ok(Response::error("Access denied", 403)?.with_headers(headers));
            }

            let secret = ctx.env.secret("SECRET_KEY")
                .map(|s| s.to_string())
                .unwrap_or_default();

            if secret.is_empty() {
                return Ok(Response::error("Server config error", 500)?.with_headers(headers));
            }

            let token = generate_token(&secret);
            let body = format!("{{\"token\":\"{}\",\"expiresIn\":{}}}", token, TOKEN_EXPIRY_MS);

            Ok(Response::ok(body)?.with_headers(headers))
        })

        // ── POST /api/send_bombing: Token check করে SMS পাঠাও ──
        .post_async("/send_bombing", |mut req, ctx| async move {
            let origin = get_origin(&req);
            let headers = cors_headers(&origin);

            // Origin check
            if !is_origin_allowed(&origin) {
                return Ok(Response::error("Access denied", 403)?.with_headers(headers));
            }

            // Token check
            let token = req.headers()
                .get("x-auth-token").unwrap_or(None)
                .unwrap_or_default();

            let secret = ctx.env.secret("SECRET_KEY")
                .map(|s| s.to_string())
                .unwrap_or_default();

            if !verify_token(&token, &secret) {
                return Ok(Response::error("Unauthorized", 401)?.with_headers(headers));
            }

            // ✅ Token valid — send_bombing logic এখানে
            let body = req.json::<serde_json::Value>().await.unwrap_or_default();
            let number = body.get("number")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();

            if number.is_empty() {
                let err = "{\"status\":\"error\",\"message\":\"Number missing\"}";
                return Ok(Response::ok(err)?.with_headers(headers));
            }

            // send_bombing.rs এর function call করো
            let result = crate::send_bombing::send(&number, &ctx.env).await;

            Ok(Response::ok(result)?.with_headers(headers))
        })

        .run(req, env)
        .await
}
