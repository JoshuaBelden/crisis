use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::{ws::Message, ws::WebSocket};

use crate::command::{handle_game_command, GameCommand};
use crate::{Client, Clients};

pub async fn receive_connection(
    ws: WebSocket,
    client_key: String,
    clients: Clients,
    mut client: Client,
) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprint!("error receiving connection: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.write().await.insert(client_key.clone(), client);

    println!("client {} connected", client_key);

    while let Some(result) = client_ws_rcv.next().await {
        let message = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprint!("error receiving message from {}: {}", client_key.clone(), e);
                break;
            }
        };
        receive_message(&client_key, message).await;
    }

    clients.write().await.remove(&client_key);
    println!("{} disconnected", client_key);
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
            eprint!("error while parsing game command message: {}", e);
            return;
        }
    };

    handle_game_command(game_command).await;
}

pub async fn broadcast_message(clients: &Clients, topic: &str, message: &str) {
    println!("broadcasting message: {}", message);
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| client.topics.contains(&topic.to_string()))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(message)));
            }
        });
}

pub async fn broadcast_debug_message(clients: &Clients, message: &str) {
    println!("broadcasting message: {}", message);
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| client.topics.contains(&"debug".to_string()))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(message)));
            }
        });
}
