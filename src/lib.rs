use worker::*;
mod send_bombing;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    send_bombing::handle(req, &env).await
}
