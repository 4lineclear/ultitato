use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::base36::{GameID, UniformID};

pub const MAX_GAMES: usize = 1000;

pub type AppArc = std::sync::Arc<AppState>;

#[derive(Default, Debug)]
pub struct AppState {
    pub waiting: Mutex<HashMap<GameID, WaitingRoom>>,
    pub rooms: (),
    pub rand_gen: UniformID,
}

impl AppState {
    #[inline]
    pub fn waiting(self: &AppState) -> MutexGuard<HashMap<GameID, WaitingRoom>> {
        self.waiting.lock().expect("Lock was poisoned")
    }
}

#[derive(Debug)]
pub struct WaitingRoom {
    pub host_id: Uuid,
    pub join_id: Uuid,
    pub host_canceller: JoinHandle<()>,
    pub host_sender: SplitSink<WebSocket, Message>,
}
