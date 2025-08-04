use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use async_std::stream::StreamExt;
use fx_hash::FxHashMap;
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    task::spawn_local,
};
use tracing::*;
use uuid::Uuid;

pub type MbMsgType = String;
pub type ConnId = Uuid;

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct MbMessageEvent;
//
// #[derive(Clone, Debug, Message)]
// #[rtype(result = "()")]
// pub struct MbMessageWrapper {
//     pub id: Uuid,
//     pub message: MbMsgType,
// }
//
// impl Actor for MbMessageEvent {
//     type Context = Context<Self>;
// }
//
// impl Handler<MbMessageWrapper> for MbMessageEvent {
//     type Result = ();
//
//     fn handle(&mut self, msg: MbMessageWrapper, _ctx: &mut Self::Context) -> Self::Result {
//         // log::trace!("SherlockMessageEvent recv message => {:?}", msg);
//         self.issue_async::<SystemBroker, _>(msg);
//     }
// }
//
// /// Define HTTP actor
// #[derive(Clone)]
// struct MessageBus {
//     event: web::Data<Addr<MbMessageEvent>>,
//     id: Uuid,
// }
//
// impl Actor for MessageBus {
//     type Context = ws::WebsocketContext<Self>;
//
//     fn started(&mut self, ctx: &mut Self::Context) {
//         self.subscribe_async::<SystemBroker, MbMessageWrapper>(ctx);
//     }
// }
//
// impl Handler<MbMessageWrapper> for MessageBus {
//     type Result = ();
//
//     fn handle(&mut self, item: MbMessageWrapper, ctx: &mut Self::Context) {
//         if item.id != self.id {
//             debug!("connection {} recv'ed a message", self.id);
//             if let Ok(json) = serde_json::to_string(&item.message) {
//                 ctx.text(json);
//             } else {
//                 warn!("could not serialize message to json string.");
//             }
//         }
//     }
// }
//
// /// Handler for ws::Message message
// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageBus {
//     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
//         match msg {
//             Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
//             Ok(ws::Message::Text(text)) => {
//                 // ctx.text(text.clone());
//                 let message = text.to_string();
//                 debug!("connection {} sent a message", self.id);
//                 self.event.do_send(MbMessageWrapper {
//                     id: self.id,
//                     message,
//                 })
//             }
//             Ok(ws::Message::Binary(_bin)) => {
//                 ctx.text("{\"response\":\"binary messages/responces are not yet implemented\"}")
//             } // ctx.binary(bin),
//             Ok(ws::Message::Close(_)) => {
//                 // ctx.close(NoneNone);
//                 ctx.&mut(None);
//                 ctx.close(None);
//
//             }
//             _ => (),
//         }
//     }
// }

// #[get("message-bus")]
// pub async fn message_bug(
//     data: web::Data<Addr<MbMessageEvent>>,
//     req: HttpRequest,
//     stream: web::Payload,
// ) -> Result<HttpResponse, Error> {
//     let id = Uuid::new_v4();
//     let resp = ws::start(MessageBus { event: data, id }, &req, stream);
//     // info!("{:?}", resp);
//     info!("ID => {id}");
//
//     resp
// }

/// A command received by the [`ChatServer`].
#[derive(Debug)]
pub enum Command {
    Connect {
        conn: ConnId,
        conn_tx: UnboundedSender<MbMsgType>,
        // res_tx: oneshot::Sender<ConnId>,
    },
    Disconnect {
        conn: ConnId,
    },
    Message {
        conn: ConnId,
        mesg: MbMsgType,
    },
}

/// Handle and command sender for chat server.
///
/// Reduces boilerplate of setting up response channels in WebSocket handlers.
#[derive(Debug, Clone)]
pub struct MbServerHandle {
    cmd_tx: UnboundedSender<Command>,
    // conn: ConnId,
}

impl MbServerHandle {
    /// Register client message sender and obtain connection ID.
    pub async fn connect(&self, conn: ConnId, conn_tx: UnboundedSender<MbMsgType>) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Connect { conn_tx, conn })
            .unwrap();
    }

    /// Broadcast message to all except sender
    pub async fn send_message(&self, conn: ConnId, msg: impl Into<MbMsgType>) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Message {
                mesg: msg.into(),
                conn,
            })
            .unwrap();
    }

    /// Unregister message sender and broadcast disconnection message to current room.
    pub fn disconnect(&self, conn: ConnId) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx.send(Command::Disconnect { conn }).unwrap();
    }
}

#[derive(Debug)]
pub struct MbServer {
    /// Map of connection IDs to their message receivers.
    sessions: FxHashMap<ConnId, UnboundedSender<MbMsgType>>,
    /// Command receiver.
    cmd_rx: UnboundedReceiver<Command>,
}

impl MbServer {
    pub fn new() -> (Self, MbServerHandle) {
        let (cmd_tx, cmd_rx) = unbounded_channel();

        (
            Self {
                sessions: FxHashMap::default(),
                cmd_rx,
            },
            MbServerHandle { cmd_tx },
        )
    }

    /// Register new session and assign unique ID to this session
    async fn connect(&mut self, id: ConnId, tx: UnboundedSender<MbMsgType>) {
        self.sessions.insert(id, tx);
    }

    /// Unregister connection from room map and broadcast disconnection message.
    async fn disconnect(&mut self, conn_id: ConnId) {
        // remove sender
        _ = self.sessions.remove(&conn_id)
    }

    async fn send_message(&mut self, conn: ConnId, mesg: impl Into<MbMsgType> + Clone) {
        for (id, channel) in self.sessions.clone().into_iter() {
            if let Err(_e) = channel.send(mesg.clone().into()) && id != conn {
                _ = self.sessions.remove(&id);
            }
        }
    }

    pub async fn run(mut self) -> std::io::Result<()> {
        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                Command::Connect { conn, conn_tx } => {
                    self.connect(conn, conn_tx).await;
                    // let _ = res_tx.send(conn_id);
                }
                Command::Disconnect { conn } => {
                    self.disconnect(conn).await;
                }
                Command::Message { conn, mesg } => {
                    self.send_message(conn, mesg).await;
                }
            }
        }

        Ok(())
    }
}

// #[get("message-bus")]
// async fn message_bus(
async fn do_message_bus(
    // data: web::Data<Addr<MbMessageEvent>>,
    // req: HttpRequest,
    // stream: web::Payload,
    chat_server: MbServerHandle,
    mut session: actix_ws::Session,
    msg_stream: actix_ws::MessageStream,
) {
    let id = Uuid::new_v4();

    // let (res, mut session, stream) = actix_ws::handle(&req, stream)?;
    let mut msg_stream = msg_stream
        .max_frame_size(128 * 1024)
        .aggregate_continuations()
        .max_continuation_size(2 * 1024 * 1024);

    let (conn_tx, mut conn_rx) = unbounded_channel();

    // unwrap: chat server is not dropped before the HTTP server
    chat_server.connect(id, conn_tx).await;

    let close_reason = {
        loop {
            select! {
                // receive messages from websocket
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        AggregatedMessage::Text(text) => {
                            // echo text message
                            // session.text(text).await.unwrap();
                            chat_server.send_message(id, text).await;
                        }

                        AggregatedMessage::Binary(_bin) => {
                            // echo binary message
                                            }
                        AggregatedMessage::Ping(_msg) => {
                            // respond to PING frame with PONG frame
                            // session.pong(&msg).await.unwrap();
                        }
                    AggregatedMessage::Close(reason) => break reason,

                        _ => {}
                    }
                }
                Some(chat_msg) = conn_rx.recv() => {
                    if let Err(e) = session.text(chat_msg).await {
                        chat_server.disconnect(id);
                        error!("failed to send message to client because: {e}");
                        break None;
                    }
                }
                else => {
                    break None;
                }
            }
        }
    };

    // attempt to close connection gracefully
    let _ = session.close(close_reason).await;
}

/// Handshake and start WebSocket handler with heartbeats.
#[get("message-bus")]
pub async fn message_bus(
    req: HttpRequest,
    stream: web::Payload,
    chat_server: web::Data<MbServerHandle>,
) -> Result<HttpResponse, Error> {
    let (res, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // spawn websocket handler (and don't await it) so that the response is returned immediately
    spawn_local(do_message_bus((**chat_server).clone(), session, msg_stream));

    Ok(res)
}
