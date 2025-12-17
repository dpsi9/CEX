use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use tokio::sync::broadcast;

use crate::server::WsState;
use crate::session::WsSession;

#[get("/ws")]
pub async fn ws_upgrade(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<WsState>,
) -> Result<HttpResponse, Error> {
    let rx: broadcast::Receiver<String> = state.broadcaster.subscribe();
    ws::start(WsSession::new(rx), &req, stream)
}
