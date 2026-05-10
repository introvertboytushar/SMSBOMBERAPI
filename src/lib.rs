use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path();
    let method = req.method();

    // CORS Headers - jate Vercel theke kono block na khai
    let mut headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", "*").unwrap();
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type").unwrap();

    // ১. OPTIONS request handle kora (CORS er jnno)
    if method == Method::Options {
        return Ok(Response::empty()?.with_headers(headers));
    }

    // ২. Front-end jodi "/get_token" call koreo fele, take fake data dao jate error na ase
    if path == "/get_token" {
        return Ok(Response::ok("{\"token\":\"no-token\",\"status\":\"ok\"}")?.with_headers(headers));
    }

    // ৩. Main Attack Path
    if path == "/send_bombing" && method == Method::Post {
        let mut req_mut = req;
        let body = req_mut.json::<serde_json::Value>().await.unwrap_or_default();
        let number = body.get("number").and_then(|v| v.as_str()).unwrap_or("").to_string();

        if number.is_empty() {
            return Ok(Response::error("Number missing", 400)?.with_headers(headers));
        }

        // Apnar bombing logic call kora
        let result = crate::send_bombing::send(&number, &env).await;
        return Ok(Response::ok(result)?.with_headers(headers));
    }

    // Onno kono path hole error
    Ok(Response::error("Not Found", 404)?.with_headers(headers))
}
