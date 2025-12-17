use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws;
use tokio::sync::broadcast;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;

pub struct WsSession {
    rx: broadcast::Receiver<String>,
}

impl WsSession {
    pub fn new(rx: broadcast::Receiver<String>) -> Self {
        Self { rx }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let stream = BroadcastStream::new(self.rx.resubscribe());
        ctx.add_stream(stream);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl StreamHandler<Result<String, BroadcastStreamRecvError>> for WsSession {
    fn handle(&mut self, msg: Result<String, BroadcastStreamRecvError>, ctx: &mut Self::Context) {
        if let Ok(payload) = msg {
            ctx.text(payload);
        }
    }
}
