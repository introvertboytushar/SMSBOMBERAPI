use worker::*;

// CORS check er jnno front-end domain
const ALLOWED_ORIGINS: &[&str] = &[
    "https://sms-bomber-it.vercel.app",
    "https://customsms-it.vercel.app",
];

fn get_origin(req: &Request) -> String {
    req.headers().get("origin").unwrap_or(None).unwrap_or_default()
}

// CORS Header function (eita sob rasta open kore dibe)
fn cors_headers(origin: &str) -> Headers {
    let mut headers = Headers::new();
    headers.set("Access-Control-Allow-Origin", if origin.is_empty() { "*" } else { &origin }).unwrap();
    headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS").unwrap();
    headers.set("Access-Control-Allow-Headers", "Content-Type, X-Auth-Token").unwrap();
    headers
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        // CORS Preflight handles
        .options("/*", |req, _ctx| {
            let origin = get_origin(&req);
            Ok(Response::empty()?.with_headers(cors_headers(&origin)))
        })
        
        // Front-end ekhon ei path-e token khujbe
        .get_async("/get_token", |req, _ctx| async move {
            let origin = get_origin(&req);
            let body = "{\"token\":\"off\",\"status\":\"ok\"}";
            Ok(Response::ok(body)?.with_headers(cors_headers(&origin)))
        })

        // Front-end ekhon ei path-e post korbe
        .post_async("/send_bombing", |mut req, ctx| async move {
            let origin = get_origin(&req);
            
            // JSON body parse
            let body = req.json::<serde_json::Value>().await.unwrap_or_default();
            let number = body.get("number").and_then(|v| v.as_str()).unwrap_or("").to_string();

            if number.is_empty() {
                return Ok(Response::error("Number missing", 400)?.with_headers(cors_headers(&origin)));
            }

            // Apnar bombing logic call
            let result = crate::send_bombing::send(&number, &ctx.env).await;
            Ok(Response::ok(result)?.with_headers(cors_headers(&origin)))
        })
        .run(req, env)
        .await
}
