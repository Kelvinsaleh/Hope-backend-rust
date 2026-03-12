use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use lazy_static::lazy_static;

pub struct WsHub {
    // userId -> Channel to send messages to that user's socket
    pub users: DashMap<String, mpsc::UnboundedSender<Message>>,
}

lazy_static! {
    pub static ref HUB: Arc<WsHub> = Arc::new(WsHub {
        users: DashMap::new(),
    });
}

impl WsHub {
    pub async fn handle_socket(user_id: String, mut socket: WebSocket) {
        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Register user in Hub
        HUB.users.insert(user_id.clone(), tx);
        tracing::info!("User {} connected to WebSocket", user_id);

        let (mut sender, mut receiver) = socket.split();

        // 1. Task to send messages FROM the hub TO the socket
        let mut send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if sender.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // 2. Task to receive messages FROM the socket (heartbeats, etc.)
        let user_id_inner = user_id.clone();
        let mut recv_task = tokio::spawn(async move {
            while let Some(Ok(_)) = receiver.next().await {
                // We could handle incoming socket messages here
            }
        });

        // Wait for either task to finish (connection close)
        tokio::select! {
            _ = (&mut send_task) => recv_task.abort(),
            _ = (&mut recv_task) => send_task.abort(),
        };

        // Cleanup
        HUB.users.remove(&user_id_inner);
        tracing::info!("User {} disconnected from WebSocket", user_id_inner);
    }

    pub fn send_to_user(user_id: &str, content: &str) {
        if let Some(tx) = HUB.users.get(user_id) {
            let _ = tx.send(Message::Text(content.to_string()));
        }
    }

    pub fn broadcast(content: &str) {
        for entry in HUB.users.iter() {
            let _ = entry.value().send(Message::Text(content.to_string()));
        }
    }
}
