use std::sync::Arc;

use crate::{
    base36::GameID,
    state::{AppArc, WaitingRoom, MAX_GAMES},
};
use axum::{
    extract::ws::{Message, WebSocket},
    Error,
};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::json;
use tokio::sync::Mutex;
use tracing::{error, info};
use uuid::Uuid;

macro_rules! json_string {
    ($items:tt) => {
        json!($items).to_string()
    };
}

pub async fn handle_register_host(socket: WebSocket, state: AppArc) {
    async fn inner(socket: WebSocket, state: AppArc) -> Result<(), (Error, Option<GameID>)> {
        info!("Creating waiting room");
        let (mut sender, receiver) = socket.split();

        if state.waiting().await.len() >= MAX_GAMES {
            error!("Max rooms hit");
            return sender
                .send(Message::Text(json_string!({
                    "status": "ServerFull"
                })))
                .await
                .map_err(|e| (e, None));
        }
        let mut game_id = GameID::new_rand(&state.rand_gen);
        loop {
            if !state.waiting().await.contains_key(&game_id) {
                break;
            }
            info!("Duplicate ID hit: {game_id:?}");
            game_id = GameID::new_rand(&state.rand_gen);
        }
        let host_id = Uuid::new_v4();

        info!(r#"Creating room, Game ID: "{game_id}", Host ID: "{host_id}""#);
        sender
            .send(Message::Text(json_string!({
                "status": "Registered",
                "game-id": game_id.to_string(),
                "host-id": host_id.to_string()
            })))
            .await
            .map_err(|e| (e, Some(game_id.clone())))?;
        info!("Host sent IDs successfully");

        state.waiting().await.insert(
            game_id.clone(),
            WaitingRoom {
                host_id,
                host_canceller: host_canceller(receiver, state.clone(), game_id),
                host_sender: sender,
            },
        );
        Ok(())
    }
    match inner(socket, state.clone()).await {
        Err((e, None)) => error!("Error sending message to host: {e}"),
        Err((e, Some(game_id))) => {
            state.waiting().await.remove(&game_id);
            panic!("Error sending message to host: {e}, Game ID '{game_id}' removed")
        }
        _ => info!("Waiting room created"),
    }
}
fn host_canceller(
    mut receiver: SplitStream<WebSocket>,
    state: AppArc,
    game_id: GameID,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match receiver.next().await {
                Some(m) => info!("Message received from Host: {m:?}"),
                None => {
                    info!("Removing game of id: {game_id}");
                    state.waiting().await.remove(&game_id);
                    return;
                }
            }
        }
    })
}
pub async fn remove_waiting((game_id, mut room): (GameID, WaitingRoom)) {
    room.host_canceller.abort();
    room.host_sender
        .send(Message::Text(json_string!({
            "status": "ServerClosed"
        })))
        .await
        .unwrap();
    info!("Waiting Room removed of ID: {game_id:?}");
}
pub async fn handle_register_join(socket: WebSocket, state: AppArc) {
    async fn inner(socket: WebSocket, state: AppArc) -> Result<(), Error> {
        let (sender, mut receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));
        let join_id = Uuid::new_v4();
        state.searching().await.insert(join_id, sender.clone());
        info!("Join attempt started with Join ID: \"{join_id}\"");
        while let Some(msg) = receiver.next().await {
            match msg? {
                Message::Text(s) => {
                    info!("Received Join message: '{s}'");
                    let game_id = s.into();
                    if let Some(WaitingRoom {
                        host_id,
                        host_canceller,
                        mut host_sender,
                    }) = state.waiting().await.remove(&game_id)
                    {
                        info!("Game of ID: '{game_id}' found");
                        sender
                            .lock()
                            .await
                            .send(Message::Text(json_string!({
                                "status": "RoomFound",
                                "game-id": game_id.to_string(),
                                "join-id": join_id.to_string(),
                            })))
                            .await?;
                        sender.lock().await.close().await?;
                        state.searching().await.remove(&join_id);
                        host_sender
                            .send(Message::Text(json_string!({
                                "status": "RoomFound",
                                "game-id": game_id.to_string(),
                                "host-id": host_id.to_string(),
                            })))
                            .await?;
                        host_sender.close().await?;
                        host_canceller.abort();
                        info!("Join & Host messages sent");
                        return Ok(());
                    } else {
                        info!("Game ID not found");
                        sender
                            .lock()
                            .await
                            .send(Message::Text(json_string!({ "status": "RoomNotFound" })))
                            .await?;
                        info!("Room not found message sent");
                    }
                }
                Message::Close(_) => {
                    state.searching().await.remove(&join_id);
                    info!("Join register attempt failed: user left before entering valid GameID");
                    break;
                }
                msg => {
                    info!("Invalid message received: {msg:?}");
                    return sender
                        .lock()
                        .await
                        .send(Message::Text(json_string!({"status": "Invalid"})))
                        .await;
                }
            }
        }
        Ok(())
    }
    match inner(socket, state).await {
        Ok(()) => info!("Join attempt end"),
        Err(e) => panic!("Error sending message to join: {e}"),
    }
}

pub async fn remove_searching(sender: (Uuid, Arc<Mutex<SplitSink<WebSocket, Message>>>)) {
    sender
        .1
        .lock()
        .await
        .send(Message::Text(json_string!({
            "status": "ServerClosed"
        })))
        .await
        .unwrap();
    info!("Searcher removed");
}
