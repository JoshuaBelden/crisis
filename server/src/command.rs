use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameCommand {
    CreateUnit {
        unit_id: String,
        position: (i32, i32),
    },
    MoveUnit {
        unit_id: String,
        destination: (i32, i32),
    },
}

pub async fn handle_game_command(command: GameCommand) {
    match command {
        GameCommand::CreateUnit { unit_id, position } => {
            println!("Creating unit {} at position {:?}", unit_id, position);
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
