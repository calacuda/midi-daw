use actix::{Actor, Addr, Context, Handler, Message, StreamHandler};
use actix_broker::{BrokerIssue, BrokerSubscribe, SystemBroker};
use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use tracing::log::*;
use uuid::Uuid;

pub type MbMsgType = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MbMessageEvent;

#[derive(Clone, Debug, Message)]
#[rtype(result = "()")]
pub struct MbMessageWrapper {
    id: Uuid,
    message: MbMsgType,
}

impl Actor for MbMessageEvent {
    type Context = Context<Self>;
}

impl Handler<MbMessageWrapper> for MbMessageEvent {
    type Result = ();

    fn handle(&mut self, msg: MbMessageWrapper, _ctx: &mut Self::Context) -> Self::Result {
        // log::trace!("SherlockMessageEvent recv message => {:?}", msg);
        self.issue_async::<SystemBroker, _>(msg);
    }
}

/// Define HTTP actor
#[derive(Clone)]
struct MessageBus {
    event: web::Data<Addr<MbMessageEvent>>,
    id: Uuid,
}

impl Actor for MessageBus {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_async::<SystemBroker, MbMessageWrapper>(ctx);
    }
}

impl Handler<MbMessageWrapper> for MessageBus {
    type Result = ();

    fn handle(&mut self, item: MbMessageWrapper, ctx: &mut Self::Context) {
        if item.id != self.id {
            debug!("connection {} recv'ed a message", self.id);
            if let Ok(json) = serde_json::to_string(&item.message) {
                ctx.text(json);
            } else {
                warn!("could not serialize message to json string.");
            }
        }
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MessageBus {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // ctx.text(text.clone());
                let message = text.to_string();
                debug!("connection {} sent a message", self.id);
                self.event.do_send(MbMessageWrapper {
                    id: self.id,
                    message,
                })
            }
            Ok(ws::Message::Binary(_bin)) => {
                ctx.text("{\"response\":\"binary messages/responces are not yet implemented\"}")
            } // ctx.binary(bin),
            _ => (),
        }
    }
}

#[get("message-bus")]
pub async fn message_bug(
    data: web::Data<Addr<MbMessageEvent>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let id = Uuid::new_v4();
    let resp = ws::start(MessageBus { event: data, id }, &req, stream);
    // info!("{:?}", resp);
    info!("ID => {id}");

    resp
}

