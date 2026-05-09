use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .options("/*", |_req, _ctx| {
            let mut headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
            headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token, X-User-ID")?;
            
            Ok(Response::empty()?.with_headers(headers))
        })
        .get_async("/api/token", |_req, _ctx| async move {
            let mut headers = Headers::new();
            headers.set("Access-Control-Allow-Origin", "*")?;
            headers.set("Content-Type", "application/json")?;
            
            // Response-ke Result-e wrap kora hoyeche 'Ok(...)' diye
            Ok(Response::ok("{\"token\": \"your_token_here\"}")?.with_headers(headers))
        })
        .run(req, env)
        .await
}
