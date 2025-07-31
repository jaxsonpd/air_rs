/// Handle the web interface for the ADS-B system.
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use axum::{routing::{get, get_service}, Json, Router};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use std::{collections::HashMap, net::SocketAddr};
use serde::Serialize;

use axum::extract::ws::{WebSocketUpgrade, WebSocket, Message};
use axum::extract::ConnectInfo;
use axum::response::IntoResponse;
use axum::routing::get as axum_get;
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr as StdSocketAddr;

use std::sync::mpsc::Receiver;

use crate::adsb::packet::AdsbPacket;
use crate::adsb::aircraft::{Aircraft, handle_aircraft_update};

const WEB_DIR: &str = "adsb_frontend/dist";

#[derive(Serialize)]
struct MyData {
    id: u32,
    message: String,
}

async fn get_data() -> Json<MyData> {
    Json(MyData {
        id: 123,
        message: "Hello from Rust backend!".to_string(),
    })
}

// Build the axum router
fn build_app(ws_tx: broadcast::Sender<String>) -> Router {
    let static_files_service = get_service(ServeDir::new(WEB_DIR));

    Router::new()
        .route("/api/data", get(get_data))
        .route("/ws", axum_get(move |ws: WebSocketUpgrade, addr: ConnectInfo<StdSocketAddr>| {
            ws_handler(ws, addr, ws_tx.clone())
        }))
        .nest_service("/", static_files_service)
}

// Run the server (async)
async fn run_server(ws_tx: broadcast::Sender<String>) {
    let app = build_app(ws_tx);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service_with_connect_info::<StdSocketAddr>())
        .await
        .unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<StdSocketAddr>,
    tx: broadcast::Sender<String>,
) -> impl IntoResponse {
    println!("WebSocket connection from {addr}");
    ws.on_upgrade(move |socket| handle_socket(socket, tx))
}

async fn handle_socket(socket: WebSocket, tx: broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    // Task to receive messages (from client → backend)
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                println!("Received from client (ignored): {}", text);
            }
        }
    });

    // Task to send messages (from backend → client)
    tokio::spawn(async move {
        loop {
            if let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    });
}

/// Handle the web interface for the ADS-B system.
/// 
/// `rx` - the receiver for ADS-B packets
pub fn web_interface_thread(rx: Receiver<AdsbPacket>) {
    // Create the Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Block on the async server run
      rt.block_on(async {
        // Broadcast channel for WebSocket messages
        let (ws_tx, _) = broadcast::channel::<String>(100);

        // Spawn the web server in the background
        let server_tx = ws_tx.clone();
        tokio::spawn(async move {
            run_server(server_tx).await;
        });

        let mut num_packets = 0;
        let mut aircrafts: HashMap<u32, Aircraft> = HashMap::new();

        loop {
            while let Ok(packet) = rx.try_recv() {
                num_packets += 1;
                let aircraft = handle_aircraft_update(packet, &mut aircrafts);
                if let Some(aircraft) = aircraft {
                    let summary = aircraft.get_summary();
                    if let Ok(json) = serde_json::to_string(&summary) {
                        // Broadcast summary to all WebSocket clients
                        println!("Broadcasting aircraft summary: {}", json);
                        let _ = ws_tx.send(json);
                    }
                }
            }

            // Avoid busy loop
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });
}