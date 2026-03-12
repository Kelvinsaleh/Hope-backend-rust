use axum::{
    extract::ws::WebSocketUpgrade,
    response::Response,
};
use crate::middleware::auth::AuthenticatedUser;
use crate::utils::ws_hub::WsHub;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user: AuthenticatedUser,
) -> Response {
    ws.on_upgrade(move |socket| WsHub::handle_socket(user.user_id, socket))
}
