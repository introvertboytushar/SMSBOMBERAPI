use worker::*;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

// ==================== ALLOWED ORIGINS ====================
const ALLOWED_ORIGINS: &[&str] = &[
    "https://sms-bomber-it.vercel.app/",   // সংশোধন: ড্যাশ ছাড়া অরিজিনাল ডোমেইন
    "https://customsms-it.vercel.app",
    "https://smsbomber.introvertboytushar.workers.dev",
];

const TOKEN_EXPIRY_MS: u64 = 2 * 60 * 1000;

// ==================== HELPERS ====================

fn get_origin(req: &Request) -> String {
    req.headers()
        .get("origin").unwrap_or(None) // lowercase "origin"
        .or_else(|| req.headers().get("referer").unwrap_or(None))
        .unwrap_or_default()
}

fn is_origin_allowed(origin: &str) -> bool {
    if origin.is_empty() { return false; }
    ALLOWED_ORIGINS.iter().any(|o| origin.starts_with(o))
}

fn cors_headers(origin: &str) -> Headers {
    let mut headers = Headers::new();
    if is_origin_allowed(origin) {
        headers.set("Access-Control-Allow-Origin", origin).unwrap();
    } else {
        // ডেভেলপমেন্টের সুবিধার জন্য চাইলে এখানে "*" দিতে পারেন
        headers.set("Access-Control-Allow-Origin", "*").unwrap();
    }
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token").unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    headers
}

// ... generate_token এবং verify_token একই থাকবে ...

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .options("/*", |req, _ctx| {
            let origin = get_origin(&req);
            let headers = cors_headers(&origin);
            Ok(Response::empty()?.with_headers(headers))
        })

        // সংশোধন: পাথ বদলানো হয়েছে যাতে index.html এর সাথে মিলে যায়
        .get_async("/get_token", |req, ctx| async move {
            let origin = get_origin(&req);
            let headers = cors_headers(&origin);

            // Origin check (Optional: আপাতত বন্ধ রাখতে পারেন যদি সমস্যা হয়)
            // if !is_origin_allowed(&origin) { return ... }

            let secret = ctx.env.secret("SECRET_KEY").map(|s| s.to_string()).unwrap_or_default();
            if secret.is_empty() {
                return Ok(Response::error("Server secret not found", 500)?.with_headers(headers));
            }

            let token = generate_token(&secret);
            let body = format!("{{\"token\":\"{}\",\"expires_at\":{}}}", token, Date::now().as_millis() + TOKEN_EXPIRY_MS);

            Ok(Response::ok(body)?.with_headers(headers))
        })

        // সংশোধন: পাথ /send_bombing করা হয়েছে
        .post_async("/send_bombing", |mut req, ctx| async move {
            let origin = get_origin(&req);
            let headers = cors_headers(&origin);

            let token = req.headers().get("x-auth-token").unwrap_or(None).unwrap_or_default();
            let secret = ctx.env.secret("SECRET_KEY").map(|s| s.to_string()).unwrap_or_default();

            if !verify_token(&token, &secret) {
                return Ok(Response::error("Unauthorized: Invalid Token", 401)?.with_headers(headers));
            }

            let body = req.json::<serde_json::Value>().await.unwrap_or_default();
            let number = body.get("number").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

            if number.is_empty() {
                return Ok(Response::error("Number missing", 400)?.with_headers(headers));
            }

            let result = crate::send_bombing::send(&number, &ctx.env).await;
            Ok(Response::ok(result)?.with_headers(headers))
        })
        .run(req, env)
        .await
}
