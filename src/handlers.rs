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

pub async fn handle_register_host(socket: WebSocket, state: AppArc) {
    async fn inner(socket: WebSocket, state: AppArc) -> Result<(), (Error, Option<GameID>)> {
        info!("Creating waiting room");
        let (mut sender, receiver) = socket.split();

        if state.waiting().await.len() >= MAX_GAMES {
            error!("Max rooms hit: {state:?}");
            return sender
                .send(Message::Text(
                    json!({
                        "status": "ServerFull"
                    })
                    .to_string(),
                ))
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
        let join_id = Uuid::new_v4();

        info!(r#"Creating room, Game ID: "{game_id}", Host ID: "{host_id}", Join ID: "{join_id}""#);
        sender
            .send(Message::Text(
                json!({
                    "status": "Registered",
                    "game-id": game_id.to_string(),
                    "host-id": host_id.to_string()
                })
                .to_string(),
            ))
            .await
            .map_err(|e| (e, Some(game_id.clone())))?;
        info!("Host ID sent successfully");

        state.waiting().await.insert(
            game_id.clone(),
            WaitingRoom {
                host_id,
                join_id,
                host_canceller: host_task(receiver, state.clone(), game_id),
                host_sender: sender,
            },
        );
        Ok(())
    }
    match inner(socket, state.clone()).await {
        Err((e, None)) => error!("Error sending message to host: {e}"),
        Err((e, Some(game_id))) => {
            info!("Removed game of ID: {game_id}");
            state.waiting().await.remove(&game_id);
            panic!("Error sending message to host: {e}")
        }
        _ => info!("Waiting room created"),
    }
}
fn host_task(
    mut receiver: SplitStream<WebSocket>,
    state: AppArc,
    game_id: GameID,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match receiver.next().await {
                Some(m) => info!("Message received: {m:?}"),
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
        .send(Message::Text(
            json!({
                "status": "ServerClosed"
            })
            .to_string(),
        ))
        .await
        .unwrap();
    info!("Room removed: {game_id:?}");
}
pub async fn remove_searching(sender: Arc<Mutex<SplitSink<WebSocket, Message>>>) {
    sender
        .lock()
        .await
        .send(Message::Text(
            json!({
                "status": "ServerClosed"
            })
            .to_string(),
        ))
        .await
        .unwrap();

    info!("Searcher removed");
}
pub async fn handle_register_join(socket: WebSocket, state: AppArc) {
    async fn inner(socket: WebSocket, state: AppArc) -> Result<(), Error> {
        info!("Join attempt start");

        let (sender, mut receiver) = socket.split();
        let sender = Arc::new(Mutex::new(sender));
        state.searching.lock().await.push(sender.clone());
        while let Some(msg) = receiver.next().await {
            match msg? {
                Message::Text(s) => {
                    info!("Received Join message: '{s}'");
                    let entry = state.waiting().await.remove(&s.clone().into());
                    if let Some(WaitingRoom {
                        host_id,
                        join_id,
                        host_canceller,
                        mut host_sender,
                    }) = entry
                    {
                        info!("Game of ID: '{s}' found");
                        sender
                            .lock()
                            .await
                            .send(Message::Text(
                                json!({
                                    "status": "RoomFound",
                                    "game-id": s.to_string(),
                                    "join-id": join_id.to_string(),
                                })
                                .to_string(),
                            ))
                            .await?;
                        sender.lock().await.close().await?;
                        host_sender
                            .send(Message::Text(
                                json!({
                                    "status": "RoomFound",
                                    "game-id": s.to_string(),
                                    "host-id": host_id.to_string(),
                                })
                                .to_string(),
                            ))
                            .await?;
                        host_sender.close().await?;
                        host_canceller.abort();
                        info!("Join & Host messages sent");
                    } else {
                        info!("Game ID not found");
                        sender
                            .lock()
                            .await
                            .send(Message::Text(
                                json!({ "status": "RoomNotFound" }).to_string(),
                            ))
                            .await?;
                        info!("Room not found message sent");
                    }
                }
                Message::Close(_) => {
                    info!("Join register attempt failed: user left before entering valid GameID");
                    break;
                }
                msg => {
                    info!("Invalid message received: {msg:?}");
                    return sender
                        .lock()
                        .await
                        .send(Message::Text(json!({"status": "Invalid"}).to_string()))
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
