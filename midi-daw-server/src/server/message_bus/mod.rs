use actix_web::{
    Error, HttpRequest, HttpResponse, get,
    web::{self, Bytes},
};
use actix_ws::AggregatedMessage;
use async_std::stream::StreamExt;
use fx_hash::FxHashMap;
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    task::spawn_local,
};
use tracing::*;
use uuid::Uuid;

// pub type MbMsgType = String;
pub type ConnId = Uuid;

#[derive(Debug, Clone)]
pub enum MbMsgType {
    Text(String),
    Bin(Bytes),
}

impl From<Bytes> for MbMsgType {
    fn from(value: Bytes) -> Self {
        Self::Bin(value)
    }
}

impl From<Vec<u8>> for MbMsgType {
    fn from(value: Vec<u8>) -> Self {
        Self::Bin(value.into())
    }
}

impl From<String> for MbMsgType {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

/// A command received by the [`ChatServer`].
#[derive(Debug)]
pub enum Command {
    Connect {
        conn: ConnId,
        conn_tx: UnboundedSender<MbMsgType>,
    },
    Disconnect {
        conn: ConnId,
    },
    Message {
        conn: ConnId,
        mesg: MbMsgType,
    },
    Binary {
        conn: ConnId,
        mesg: Vec<u8>,
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
    pub fn send_message(&self, conn: ConnId, msg: impl ToString) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Message {
                mesg: msg.to_string().into(),
                conn,
            })
            .unwrap();
    }

    /// Broadcast binary data to all except sender
    pub fn send_binary(&self, conn: ConnId, msg: Bytes) {
        // unwrap: chat server should not have been dropped
        self.cmd_tx
            .send(Command::Binary {
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
            if let Err(_e) = channel.send(mesg.clone().into())
                && id != conn
            {
                _ = self.sessions.remove(&id);
            }
        }
    }

    async fn send_binary(&mut self, conn: ConnId, mesg: Bytes) {
        for (id, channel) in self.sessions.clone().into_iter() {
            if let Err(_e) = channel.send(mesg.clone().into())
                && id != conn
            {
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
                Command::Binary { conn, mesg } => {
                    self.send_binary(conn, mesg.into()).await;
                }
            }
        }

        Ok(())
    }
}

async fn do_message_bus(
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
                            // text message
                            // session.text(text).await.unwrap();
                            chat_server.send_message(id, text);
                        }

                        AggregatedMessage::Binary(bin) => {
                            // binary message
                            chat_server.send_binary(id, bin);
                        }
                        AggregatedMessage::Ping(_msg) => {
                            // respond to PING frame with PONG frame
                            // session.pong(&msg).await.unwrap();
                        }
                    AggregatedMessage::Close(reason) => break reason,

                        _ => {}
                    }
                }
                // send to connected clients
                Some(chat_msg) = conn_rx.recv() => {
                    match chat_msg {
                        MbMsgType::Text(chat_msg) => if let Err(e) = session.text(chat_msg).await {
                            chat_server.disconnect(id);
                            error!("failed to send message to client because: {e}");
                            break None;
                        }
                        MbMsgType::Bin(chat_msg) => if let Err(e) = session.binary(chat_msg).await {
                            chat_server.disconnect(id);
                            error!("failed to send message to client because: {e}");
                            break None;
                        }
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
