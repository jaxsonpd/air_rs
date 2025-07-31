/// Handle the web interface for the ADS-B system.
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use axum::{routing::{get, get_service}, Json, Router};
use tower_http::services::ServeDir;
use std::{collections::HashMap, hash::Hash, net::SocketAddr};
use serde::Serialize;

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
fn build_app() -> Router {
    let static_files_service = get_service(ServeDir::new(WEB_DIR));

    Router::new()
        .route("/api/data", get(get_data))
        .nest_service("/", static_files_service)
}

// Run the server (async)
async fn run_server() {
    let app = build_app();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on http://{}", addr);

    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}

/// Handle the web interface for the ADS-B system.
/// 
/// `rx` - the receiver for ADS-B packets
pub fn web_interface_thread(rx: Receiver<AdsbPacket>) {
    // Create the Tokio runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Block on the async server run
    rt.block_on(async {
        // Spawn the web server in the background
        tokio::spawn(async {
            run_server().await;
        });

        let mut num_packets = 0;
        let mut aircrafts: HashMap<u32, Aircraft> = HashMap::new();
        
        loop {
            while let Ok(packet) = rx.try_recv() {
                num_packets += 1;
                let aircraft = handle_aircraft_update(packet, &mut aircrafts);
                println!("Received packet for aircraft: {:?}", aircraft);
            }
        }
    });

    
}