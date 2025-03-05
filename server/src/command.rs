use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GameCommand {
    CreateUnit {
        unit_id: usize,
        position: (i32, i32),
    },
    MoveUnit {
        unit_id: usize,
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
    // clients
    // .read()
    // .await
    // .iter()
    // /*
    // If the body contains a user_id, we want to send the message to all clients but the sender.
    // Otherwise, we want to send the message to all clients that have subscribed to the topic.
    // */
    // .filter(|(_, client)| match publish_request.user_id {
    //     Some(user_id) => client.user_id != user_id,
    //     None => true,
    // })
    // .filter(|(_, client)| client.topics.contains(&publish_request.topic))
    // .for_each(|(_, client)| {
    //     if let Some(sender) = &client.sender {
    //         let game_command = serde_json::to_string(&publish_request.game_command).unwrap();
    //         let _ = sender.send(Ok(Message::text(game_command)));
    //     }
    // });
}
