mod command;
mod handler;
mod socket;

use std::sync::Arc;
use std::{collections::HashMap, convert::Infallible};
use tokio::sync::{mpsc, RwLock};
use warp::Filter;
use warp::{ws::Message, Rejection};

use handler::{health_handler, register_handler, socket_handler};

pub type Clients = Arc<RwLock<HashMap<String, Client>>>;
pub type Result<T> = std::result::Result<T, Rejection>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    let health_route = warp::path!("health").and_then(health_handler);

    let register_routes = warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(register_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(socket_handler);

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}
