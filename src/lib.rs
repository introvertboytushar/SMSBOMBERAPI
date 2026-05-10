use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    let method = req.method();

    // CORS Headers - এটা সব ব্লক ছাড়িয়ে দেবে
    let mut headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", "*").unwrap();
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token").unwrap();

    // ১. প্রি-ফ্লাইট রিকোয়েস্ট (OPTIONS)
    if method == Method::Options {
        return Ok(Response::empty()?.with_headers(headers));
    }

    // ২. টোকেন পাথ চেক
    if path == "/get_token" {
        let body = "{\"token\":\"off\",\"status\":\"ok\"}";
        return Ok(Response::ok(body)?.with_headers(headers));
    }

    // ৩. অ্যাটাক পাথ চেক
    if path == "/send_bombing" && method == Method::Post {
        let mut req_mut = req;
        let body = req_mut.json::<serde_json::Value>().await.unwrap_or_default();
        let number = body.get("number").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if number.is_empty() {
            return Ok(Response::error("Number missing", 400)?.with_headers(headers));
        }

        // আপনার আসল বোমা মারার ফাংশন
        let result = crate::send_bombing::send(&number, &env).await;
        return Ok(Response::ok(result)?.with_headers(headers));
    }

    // ভুল পাথে আসলে এটা দেখাবে
    Ok(Response::error(format!("Path {} not found", path), 404)?.with_headers(headers))
}
