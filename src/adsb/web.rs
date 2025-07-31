/// Handle the web interface for the ADS-B system.
/// 
/// Author: Jack Duignan (JackpDuignan@gmail.com)

use std::fs;
use std::path::Path;
use tiny_http::{Server, Response};

use std::sync::mpsc::Receiver;

use crate::adsb::packet::AdsbPacket;

const WEB_DIR: &str = "adsb_frontend/dist";

/// Start the web interface
/// 
fn start_web_server() {
     let server = Server::http("0.0.0.0:8080").unwrap();

    println!("Server started at http://localhost:8080");

    for request in server.incoming_requests() {
        let url = request.url();
        let path = if url == "/" {
            "adsb_frontend/dist/index.html"
        } else {
            &format!("adsb_frontend/dist{}", url)
        };

        let path = Path::new(path);

        let response = if path.exists() && path.is_file() {
            let content = fs::read(path).unwrap();
            let mime_type = mime_guess::from_path(path).first_or_octet_stream();
            Response::from_data(content).with_header(
                tiny_http::Header::from_bytes(&b"Content-Type"[..], mime_type.essence_str()).unwrap()
            )
        } else {
            Response::from_string("404 Not Found").with_status_code(404)
        };

        let _ = request.respond(response);
    }
}

/// Handle the web interface for the ADS-B system.
/// 
/// `rx` - the receiver for ADS-B packets
pub fn web_interface_thread(rx: Receiver<AdsbPacket>) {
    start_web_server();
    loop {
        match rx.recv() {
            Ok(packet) => {
                // Process the packet and update the web interface
                // This could involve sending data to a WebSocket or updating a shared state
                println!("Received packet: {:?}", packet);
            },
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
                break;
            }
        }
    }
}