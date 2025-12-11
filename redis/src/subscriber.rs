use futures_util::StreamExt;
use redis_rs::aio::PubSub;
use shared::CexError;
use tokio::sync::mpsc::{self, Receiver};

pub struct RedisSubscriber {
    rx: Receiver<Result<String, CexError>>,
}

impl RedisSubscriber {
    pub fn new(mut pubsub: PubSub) -> Self {
        let (tx, rx) = mpsc::channel(256);
        tokio::spawn(async move {
            let mut messages = pubsub.on_message();
            while let Some(msg) = messages.next().await {
                let payload: Result<String, _> = msg.get_payload();
                let send_res = match payload {
                    Ok(data) => tx.send(Ok(data)).await,
                    Err(err) => {
                        tx.send(Err(CexError::Redis(format!("payload decode error: {err}"))))
                            .await
                    }
                };
                if send_res.is_err() {
                    break;
                }
            }
        });
        Self { rx }
    }

    pub fn into_channel(self) -> Receiver<Result<String, CexError>> {
        self.rx
    }
}
