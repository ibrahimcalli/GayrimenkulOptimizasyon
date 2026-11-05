mod auth;
mod db;
mod handlers;
mod state;

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::routing::{delete, get, post, put};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use state::AppState;

fn data_dir() -> PathBuf {
    // Sunucu verisini çalıştırılabilir dosyanın yanındaki "data" klasöründe tut.
    // Bu sayede uygulama taşınabilir (portable) olur.
    if let Ok(dir) = std::env::var("GAYRIMENKUL_DATA") {
        return PathBuf::from(dir);
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("data")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let base = data_dir();
    let upload_dir = base.join("uploads");
    std::fs::create_dir_all(&upload_dir).expect("uploads klasörü oluşturulamadı");

    let db_path = base.join("gayrimenkul.db");
    let conn = rusqlite::Connection::open(&db_path).expect("Veritabanı açılamadı");
    db::init(&conn).expect("Veritabanı şeması oluşturulamadı");

    let port: u16 = std::env::var("GAYRIMENKUL_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Wi-Fi / kablolu ağ üzerinden otomatik sunucu keşfi
    start_discovery(port, hostname());

    let state = AppState::new(conn, upload_dir, base.clone());

    let api = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        // auth
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/logout", post(handlers::logout))
        .route("/api/me", get(handlers::me).put(handlers::update_profile))
        .route("/api/me/password", post(handlers::change_password))
        .route("/api/me/photo", post(handlers::upload_my_photo))
        // users
        .route("/api/users", get(handlers::list_users).post(handlers::create_user))
        .route("/api/users/:id", put(handlers::update_user))
        .route("/api/users/:id/toggle", post(handlers::toggle_user))
        // portfoy
        .route("/api/portfolios", get(handlers::list_portfolios).post(handlers::create_portfolio))
        .route(
            "/api/portfolios/:id",
            get(handlers::get_portfolio)
                .put(handlers::update_portfolio)
                .delete(handlers::delete_portfolio),
        )
        .route("/api/portfolios/:id/images", post(handlers::upload_portfolio_images))
        .route(
            "/api/portfolios/:id/images/:filename",
            delete(handlers::delete_portfolio_image),
        )
        .route("/api/portfolios/:id/restore", post(handlers::restore_portfolio))
        // crm
        .route("/api/crm", get(handlers::list_crm).post(handlers::create_crm))
        .route(
            "/api/crm/:id",
            get(handlers::get_crm).put(handlers::update_crm).delete(handlers::delete_crm),
        )
        .route("/api/crm/:id/restore", post(handlers::restore_crm))
        .route("/api/crm/:id/photo", post(handlers::upload_crm_photo))
        // gorevler
        .route("/api/tasks", get(handlers::list_tasks).post(handlers::create_task))
        .route("/api/tasks/:id/toggle", post(handlers::toggle_task))
        .route("/api/tasks/:id", delete(handlers::delete_task))
        // hedefler
        .route("/api/goals", get(handlers::list_goals).post(handlers::create_goal))
        .route("/api/goals/:id", delete(handlers::delete_goal))
        // vardiya planı ve mesai onayı
        .route("/api/shift/plans", get(handlers::list_shift_plans))
        .route("/api/shift/plan", post(handlers::upsert_shift_plan))
        .route("/api/shift/attendance", get(handlers::list_attendance))
        .route("/api/shift/attendance", post(handlers::upsert_attendance))
        .route("/api/shift/attendance/:id/toggle-approve", post(handlers::toggle_attendance))
        // bildirim
        .route("/api/notifications", get(handlers::list_notifications))
        .route("/api/notifications/read-all", post(handlers::read_all_notifications))
        // ayarlar
        .route("/api/settings", get(handlers::get_settings).put(handlers::update_settings))
        // yedekleme (yalnızca admin)
        .route("/api/backup", get(handlers::backup))
        // dosyalar
        .route("/api/files/:name", get(handlers::serve_file))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    print_banner(port, &db_path);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Port {port} dinlenemedi: {e}"));
    axum::serve(listener, api).await.expect("Sunucu çalıştırılamadı");
}

/// Ağdaki istemcilerin sunucuyu otomatik bulabilmesi için UDP keşif servisini başlatır.
/// Hem kablolu LAN hem de Wi-Fi üzerinden çalışır.
fn start_discovery(http_port: u16, name: String) {
    use common::discovery::{DiscoveryReply, REQUEST, UDP_PORT};
    std::thread::spawn(move || {
        let sock = match std::net::UdpSocket::bind(("0.0.0.0", UDP_PORT)) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(
                    "Keşif servisi başlatılamadı (UDP {UDP_PORT}): {e}. Otomatik bulma çalışmayabilir."
                );
                return;
            }
        };
        let _ = sock.set_broadcast(true);
        let reply = DiscoveryReply {
            name,
            port: http_port,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let payload = serde_json::to_vec(&reply).unwrap_or_default();
        let mut buf = [0u8; 256];
        loop {
            match sock.recv_from(&mut buf) {
                Ok((n, from)) => {
                    if &buf[..n] == REQUEST {
                        let _ = sock.send_to(&payload, from);
                    }
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(200)),
            }
        }
    });
}

/// Bilgisayar adını (hostname) döndürür.
fn hostname() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::fs::read_to_string("/etc/hostname")
                .ok()
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "Gayrimenkul360-Sunucu".to_string())
}

fn print_banner(port: u16, db_path: &std::path::Path) {
    println!("\n============================================================");
    println!("  GAYRİMENKUL360 - Ofis Sunucusu çalışıyor");
    println!("============================================================");
    println!("  Yerel adres  : http://127.0.0.1:{port}");
    for ip in local_ips() {
        println!("  Ağ adresi    : http://{ip}:{port}   <-- diğer bilgisayarlar bunu kullanır");
    }
    println!("  Veritabanı   : {}", db_path.display());
    println!("  Otomatik bulma: açık (Wi-Fi + kablolu ağ, UDP 8085)");
    println!("  Varsayılan giriş: admin / admin123  (ilk girişte değiştirin!)");
    println!("============================================================\n");
}

/// Yerel ağ (LAN) IP adreslerini bulmaya çalışır.
fn local_ips() -> Vec<String> {
    use std::net::UdpSocket;
    let mut ips = Vec::new();
    // Özel yönlendirilebilir bir adrese "bağlanarak" yerel arayüz IP'sini öğren.
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                ips.push(addr.ip().to_string());
            }
        }
    }
    if ips.is_empty() {
        ips.push("SUNUCU_IP".to_string());
    }
    ips
}
