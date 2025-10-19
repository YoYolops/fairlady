// The server dispatcher module does something very similar to client's
// but at the same time is the complete inverse.
// This module is responsible for dispatching incoming data to server internal hadlers

use tokio;

pub async fn spawn_dispatcher() {
    tokio::spawn(async move {

    });
}