use crate::model::AvailableRooms;
use crate::websockets::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};
use actix::prelude::{Actor, Context, Handler, Recipient};
use actix_web::web::Data;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::model::Acess::Private;
use crate::model::Status::Sucess;

use super::EchoAvailableRoomsLobby;


type Socket = Recipient<WsMessage>;

#[derive(Serialize)]
struct RoomsState {
    available_rooms_state: Data<Arc<Mutex<Vec<AvailableRooms>>>>
}

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>,          //self id to self
    rooms: HashMap<Uuid, HashSet<Uuid>>,      //room id  to list of users id
    rooms_state: RoomsState
}


impl Lobby {
    pub fn new(available_rooms_state: Data<Arc<Mutex<Vec<AvailableRooms>>>>) -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            rooms: HashMap::new(), 
            rooms_state:RoomsState{available_rooms_state} 
        }
    }
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient
                .do_send(WsMessage(message.to_owned()));
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}
impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // create a room if necessary, and then add the id to it
        self.rooms
            .entry(msg.lobby_id)
            .or_insert_with(HashSet::new).insert(msg.self_id);
        // store the address
        self.sessions.insert(
            msg.self_id,
            msg.addr,
        );
        //send private message to build the initial screen
        let serialized_rooms = serde_json::to_string(&self.rooms_state.available_rooms_state).unwrap();
        self.send_message(serialized_rooms.as_str(), &msg.self_id);
    }
}
/// Handler for Disconnect message.
impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
 
            if let Some(lobby) = self.rooms.get_mut(&msg.room_id) {
                if lobby.len() > 1 {
                    lobby.remove(&msg.id);
                } else {
                    //only one in the lobby, remove it entirely
                    self.rooms.remove(&msg.room_id);
                }
            }
        }
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _: &mut Context<Self>) -> Self::Result {
        
        let notification_serialized = serde_json::to_string(&msg.notification).unwrap();

        if msg.notification.acess == Private {
            return self.send_message(& notification_serialized, &msg.id);
        }
        if msg.notification.status== Sucess{
            self.send_message(& notification_serialized, &msg.id);
        } 
        let a = serde_json::to_string(&msg.notification.data).unwrap();
        self.rooms
            .get(&msg.room_id)
            .unwrap()
            .iter()
            .filter(|conn_id| *conn_id.to_owned() != msg.id)
            .for_each(|client| {
                self.send_message(&a, client)
            });  
    }
}

impl Handler<EchoAvailableRoomsLobby> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: EchoAvailableRoomsLobby, _: &mut Context<Self>) -> Self::Result {
        let serialized_rooms = serde_json::to_string(&self.rooms_state.available_rooms_state).unwrap();
        self.rooms
            .get(&msg.lobby_id)
            .unwrap()
            .iter()
            .for_each(|client|{
                self.send_message(serialized_rooms.as_str(), client)
            });
    }
}

