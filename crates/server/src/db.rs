use rand::Rng;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};

pub fn now() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn new_salt() -> String {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 16] = rng.gen();
    hex::encode(bytes)
}

pub fn new_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Veritabanı şemasını oluşturur ve varsayılan admin kullanıcısını ekler.
pub fn init(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            salt TEXT NOT NULL,
            full_name TEXT NOT NULL DEFAULT '',
            role TEXT NOT NULL DEFAULT 'agent',
            phone TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            photo TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS crm (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            full_name TEXT NOT NULL,
            phone TEXT NOT NULL DEFAULT '',
            email TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',
            owner_type TEXT NOT NULL DEFAULT 'Sahip',
            notes TEXT NOT NULL DEFAULT '',
            created_by INTEGER NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS portfolios (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            listing_type TEXT NOT NULL DEFAULT 'satilik',
            category TEXT NOT NULL DEFAULT 'konut',
            status TEXT NOT NULL DEFAULT 'taslak',
            price REAL NOT NULL DEFAULT 0,
            currency TEXT NOT NULL DEFAULT 'TRY',
            area_m2 REAL NOT NULL DEFAULT 0,
            room_count TEXT NOT NULL DEFAULT '',
            building_age TEXT NOT NULL DEFAULT '',
            floor TEXT NOT NULL DEFAULT '',
            total_floors TEXT NOT NULL DEFAULT '',
            heating TEXT NOT NULL DEFAULT '',
            furnished INTEGER NOT NULL DEFAULT 0,
            il TEXT NOT NULL DEFAULT 'Muğla',
            ilce TEXT NOT NULL DEFAULT '',
            mahalle TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',
            latitude REAL NOT NULL DEFAULT 0,
            longitude REAL NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT '',
            owner_crm_id INTEGER,
            created_by INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS portfolio_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            portfolio_id INTEGER NOT NULL,
            filename TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            assigned_to INTEGER,
            assigned_by INTEGER NOT NULL,
            due_at TEXT NOT NULL DEFAULT '',
            priority TEXT NOT NULL DEFAULT 'normal',
            done INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            scope TEXT NOT NULL DEFAULT 'user',
            user_id INTEGER,
            title TEXT NOT NULL,
            metric TEXT NOT NULL DEFAULT '',
            target_count INTEGER NOT NULL DEFAULT 0,
            period_start TEXT NOT NULL DEFAULT '',
            period_end TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS shifts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            clock_in TEXT NOT NULL,
            clock_out TEXT,
            note TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            kind TEXT NOT NULL DEFAULT 'info',
            read INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            show_name INTEGER NOT NULL DEFAULT 1,
            show_phone INTEGER NOT NULL DEFAULT 1,
            show_email INTEGER NOT NULL DEFAULT 1,
            show_price INTEGER NOT NULL DEFAULT 1,
            show_address INTEGER NOT NULL DEFAULT 1,
            show_description INTEGER NOT NULL DEFAULT 1,
            show_cover_image INTEGER NOT NULL DEFAULT 1,
            header_text TEXT NOT NULL DEFAULT 'GAYRİMENKUL360',
            footer_text TEXT NOT NULL DEFAULT 'Bu tanıtım belgesi Gayrimenkul360 tarafından oluşturulmuştur.'
        );
        "#,
    )?;

    // Varsayılan ayarlar
    conn.execute(
        "INSERT OR IGNORE INTO settings (id) VALUES (1)",
        [],
    )?;

    // Varsayılan admin yoksa oluştur: admin / admin123
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users WHERE role='admin'", [], |r| r.get(0))?;
    if count == 0 {
        let salt = new_salt();
        let hash = hash_password("admin123", &salt);
        conn.execute(
            "INSERT INTO users (username, password_hash, salt, full_name, role, phone, email, active, created_at)
             VALUES (?1, ?2, ?3, ?4, 'admin', '', '', 1, ?5)",
            params!["admin", hash, salt, "Sistem Yöneticisi", now()],
        )?;
        tracing::warn!("Varsayılan admin oluşturuldu: admin / admin123 (ilk girişten sonra değiştirin!)");
    }
    Ok(())
}
