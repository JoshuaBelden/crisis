use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{ws::Message, ws::WebSocket};

use crate::command::{handle_game_command, GameCommand};
use crate::{Client, Clients};

pub async fn receive_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprint!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.write().await.insert(id.clone(), client);

    println!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let message = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprint!("error receiving ws message for id={}: {}", id.clone(), e);
                break;
            }
        };
        receive_message(&id, message).await;
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn receive_message(id: &str, message: Message) {
    println!("received message from {}: {:?}", id, message);
    let message = match message.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let game_command: GameCommand = match serde_json::from_str(&message) {
        Ok(req) => req,
        Err(e) => {
            eprint!("error while parsing message to game command: {}", e);
            return;
        }
    };

    handle_game_command(game_command).await;
}

pub async fn broadcast_message(clients: &Clients, message: &str) {
    let clients = clients.read().await;
    for (_, client) in clients.iter() {
        if client.topics.contains(&"debug".to_string()) {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(message)));
            }
        }
    }
}
