use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::socket::broadcast_message;
use crate::Clients;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GameCommand {
    #[serde(rename_all = "camelCase")]
    CreateUnit {
        unit_id: String,
        position: (i32, i32),
    },
    #[serde(rename_all = "camelCase")]
    MoveUnit {
        unit_id: String,
        destination: (i32, i32),
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum WorldEvent {
    PlayerJoined,
    PlayerLeft,
    #[serde(rename_all = "camelCase")]
    UnitCreated {
        unit_id: String,
        position: (i32, i32),
    },
    #[serde(rename_all = "camelCase")]
    UnitMoved {
        unit_id: String,
        destination: (i32, i32),
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameCommandRequest {
    player_id: String,
    game_command: GameCommand,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct WorldEventResponse {
    player_id: String,
    world_event: WorldEvent,
}

pub async fn handle_game_command(game_command_request: GameCommandRequest, clients: &Clients) {
    match game_command_request.game_command {
        GameCommand::CreateUnit { unit_id, position } => {
            println!("Creating unit {} at position {:?}", unit_id, position);

            let world_event_response = WorldEventResponse {
                player_id: game_command_request.player_id,
                world_event: WorldEvent::UnitCreated { unit_id, position },
            };

            let world_event_response_str = to_string(&world_event_response).unwrap();

            broadcast_message(clients, "world-events", &world_event_response_str).await;
        }
        GameCommand::MoveUnit {
            unit_id,
            destination,
        } => {
            println!("Moving unit {} to {:?}", unit_id, destination);
        }
    }
    // after a game update, broadcast the update to all users
    // broadcast_message(clients, "game", "game update").await;
}
