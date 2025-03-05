use futures::{FutureExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::HashMap, convert::Infallible};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, ws::WebSocket, Filter, Rejection, Reply};

type Clients = Arc<RwLock<HashMap<String, Client>>>;
type Result<T> = std::result::Result<T, Rejection>;

#[derive(Serialize, Deserialize, Debug)]
enum GameCommand {
    CreateUnit {
        unit_id: usize,
        position: (i32, i32),
    },
    MoveUnit {
        unit_id: usize,
        destination: (i32, i32),
    },
}

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Deserialize, Debug)]
struct RegisterRequest {
    user_id: usize,
    topic: String,
}

#[derive(Serialize, Debug)]
struct RegisterResponse {
    url: String,
}

#[derive(Deserialize, Debug)]
struct PublishRequest {
    topic: String,
    user_id: Option<usize>,
    game_command: GameCommand,
}

async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<impl Reply> {
    let user_id = body.user_id;
    let topic = body.topic;
    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), user_id, topic, clients).await;
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    }))
}

async fn publish_handler(body: PublishRequest, clients: Clients) -> Result<impl Reply> {
    clients
        .read()
        .await
        .iter()
        /*
        If the body contains a user_id, we want to send the message to all clients but the sender.
        Otherwise, we want to send the message to all clients that have subscribed to the topic.
        */
        .filter(|(_, client)| match body.user_id {
            Some(user_id) => client.user_id != user_id,
            None => true,
        })
        .filter(|(_, client)| client.topics.contains(&body.topic))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let game_command = serde_json::to_string(&body.game_command).unwrap();
                let _ = sender.send(Ok(Message::text(game_command)));
            }
        });

    broadcast_message(&clients, "A published message was handled").await;

    Ok(StatusCode::OK)
}

async fn socket_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => {
            Ok(ws.on_upgrade(move |socket| handle_client_connection(socket, id, clients, c)))
        }
        None => Err(warp::reject::not_found()),
    }
}

async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}

async fn register_client(id: String, user_id: usize, topic: String, clients: Clients) {
    clients.write().await.insert(
        id,
        Client {
            user_id,
            topics: vec![topic],
            sender: None,
        },
    );

    broadcast_message(&clients, "A new client was registered").await;
}

async fn handle_client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
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
        handle_client_message(&id, message).await;
    }

    clients.write().await.remove(&id);
    println!("{} disconnected", id);
}

async fn handle_client_message(id: &str, message: Message) {
    println!("received message from {}: {:?}", id, message);
    let message = match message.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let publish_request: PublishRequest = match serde_json::from_str(&message) {
        Ok(req) => req,
        Err(e) => {
            eprint!("error while parsing message to game command: {}", e);
            return;
        }
    };

    handle_game_command(publish_request.game_command).await;
}

async fn handle_game_command(command: GameCommand) {
    match command {
        GameCommand::CreateUnit { unit_id, position } => {
            println!("Creating unit {} at position {:?}", unit_id, position);
        }
        GameCommand::MoveUnit {
            unit_id,
            destination,
        } => {
            println!(
                "Moving unit {} to {:?}",
                unit_id, destination
            );
        }
    }
}

async fn broadcast_message(clients: &Clients, message: &str) {
    let clients = clients.read().await;
    for (_, client) in clients.iter() {
        if client.topics.contains(&"debug".to_string()) {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(message)));
            }
        }
    }
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    let health_route = warp::path!("health").and_then(health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(register_handler);

    let publish_route = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(publish_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(socket_handler);

    let routes = health_route
        .or(register_routes)
        .or(publish_route)
        .or(ws_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
