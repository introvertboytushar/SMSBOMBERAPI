use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .options("/*", |_req, _ctx| {
            // Preflight request handle korar jonno header
            let mut headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token, X-User-ID")?;
            
            Ok(Response::empty()?.with_headers(headers))
        })
        .get_async("/api/token", |req, ctx| async move {
            // Token logic ekhane thakbe
            let mut headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            
            // Response-er sathe headers pathano
            Response::ok("{\"token\": \"your_token_here\"}")?.with_headers(headers)
        })
        // Onno endpoint gulo (send_bombing) ekhane thakbe
        .run(req, env).await
}
