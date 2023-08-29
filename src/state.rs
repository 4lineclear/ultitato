use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
use tokio::{
    sync::{Mutex, MutexGuard},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::base36::{GameID, UniformID};

pub const MAX_GAMES: usize = 1000;

pub type AppArc = std::sync::Arc<AppState>;

pub type Searching = Mutex<SearchingInner>;
pub type SearchingInner = HashMap<Uuid, Arc<Mutex<SplitSink<WebSocket, Message>>>>;
pub type Waiting = Arc<Mutex<HashMap<GameID, WaitingRoom>>>;

#[derive(Default, Debug)]
pub struct AppState {
    pub waiting: Waiting,
    pub searching: Searching,
    pub rooms: (),
    pub rand_gen: UniformID,
}

impl AppState {
    #[inline]
    pub async fn waiting(&self) -> MutexGuard<HashMap<GameID, WaitingRoom>> {
        self.waiting.lock().await
    }
    #[inline]
    pub async fn searching(&self) -> MutexGuard<SearchingInner> {
        self.searching.lock().await
    }
}

#[derive(Debug)]
pub struct WaitingRoom {
    pub host_id: Uuid,
    pub host_canceller: JoinHandle<()>,
    pub host_sender: SplitSink<WebSocket, Message>,
}
