//! Ofis ağı (kablolu LAN veya Wi-Fi) üzerinde sunucu otomatik keşfi.
//!
//! Sunucu, UDP üzerinden gelen keşif isteğine cevap verir. İstemci, yayın
//! (broadcast) adresine istek gönderir ve cevap veren sunucuların IP'sini
//! öğrenir. Böylece çalışanlar sunucunun IP adresini elle yazmak zorunda kalmaz.
//! Bu yöntem hem Ethernet hem de Wi-Fi bağlantılarında çalışır.

use serde::{Deserialize, Serialize};

/// Keşif için kullanılan UDP portu.
pub const UDP_PORT: u16 = 8085;

/// Keşif isteği imzası (istemci -> yayın).
pub const REQUEST: &[u8] = b"GAYRIMENKUL360_DISCOVER_V1";

/// Sunucunun keşif cevabı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReply {
    pub name: String,
    pub port: u16,
    pub version: String,
}

/// İstemcinin bulduğu bir sunucu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredServer {
    pub ip: String,
    pub port: u16,
    pub name: String,
    pub version: String,
}

impl DiscoveredServer {
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.ip, self.port)
    }

    pub fn label(&self) -> String {
        format!("{}  ({}:{})", self.name, self.ip, self.port)
    }
}
