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

/// Mevcut veritabanına sonradan eklenen kolonları güvenli şekilde ekler.
///
/// Idempotenttir: her kolon ALTER edilmeden önce `PRAGMA table_info` ile
/// varlığı kontrol edilir; mevcutsa ekleme adımı atlanır. Böylece uygulama
/// her başlangıçta güvenle çalıştırılabilir.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    // Bozuk göç (kolon adı eksik ALTER) nedeniyle oluşmuş olabilecek çöp kolonu temizle
    drop_column_if_exists(conn, "settings", "INTEGER")?;
    drop_column_if_exists(conn, "settings", "REAL")?;
    drop_column_if_exists(conn, "settings", "TEXT")?;

    // (tablo, kolon adı, kolon tanımı) — ALTER'da kolon adı ayrı ve zorunludur
    let columns: &[(&str, &str, &str)] = &[
        ("settings", "val_title_min", "INTEGER NOT NULL DEFAULT 3"),
        ("settings", "val_phone_digits", "INTEGER NOT NULL DEFAULT 10"),
        ("settings", "val_email_required", "INTEGER NOT NULL DEFAULT 0"),
        ("settings", "val_phone_required", "INTEGER NOT NULL DEFAULT 0"),
        ("settings", "val_desc_min", "INTEGER NOT NULL DEFAULT 0"),
        ("settings", "val_price_min", "REAL NOT NULL DEFAULT 0"),
        ("settings", "val_area_min", "REAL NOT NULL DEFAULT 0"),
        ("settings", "default_theme", "TEXT NOT NULL DEFAULT 'system'"),
        ("settings", "default_zoom", "REAL NOT NULL DEFAULT 1.0"),
        ("crm", "photo", "TEXT"),
        ("crm", "is_deleted", "INTEGER NOT NULL DEFAULT 0"),
        ("crm", "deleted_by_user_id", "INTEGER"),
        ("portfolios", "is_deleted", "INTEGER NOT NULL DEFAULT 0"),
        ("portfolios", "deleted_by_user_id", "INTEGER"),
        ("tasks", "is_deleted", "INTEGER NOT NULL DEFAULT 0"),
        ("shift_plans", "weekdays", "TEXT NOT NULL DEFAULT ''"),
        ("portfolios", "area_net", "REAL NOT NULL DEFAULT 0"),
        ("portfolios", "ada_parsel", "TEXT NOT NULL DEFAULT ''"),
    ];
    for (table, name, ddl) in columns {
        if column_exists(conn, table, name) {
            continue; // kolon zaten mevcut, atla
        }
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN \"{name}\" {ddl}"),
            [],
        )?;
        tracing::info!("{table} tablosuna kolon eklendi: {name}");
    }
    Ok(())
}

/// Kolonun tabloda var olup olmadığını `PRAGMA table_info` ile kontrol eder.
fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info(?1) WHERE name = ?2",
        params![table, column],
        |r| r.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
}

/// Tablodan kolonu (varsa) güvenle düşürür. Bozuk/çöp kolon temizliği içindir.
fn drop_column_if_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<()> {
    if column_exists(conn, table, column) {
        conn.execute(&format!("ALTER TABLE {table} DROP COLUMN \"{column}\""), [])?;
        tracing::warn!("{table} tablosundaki bozuk '{column}' kolonu temizlendi");
    }
    Ok(())
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
            created_at TEXT NOT NULL,
            photo TEXT,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_by_user_id INTEGER
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
            area_net REAL NOT NULL DEFAULT 0,
            ada_parsel TEXT NOT NULL DEFAULT '',
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
            updated_at TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_by_user_id INTEGER
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
            created_at TEXT NOT NULL,
            is_deleted INTEGER NOT NULL DEFAULT 0,
            deleted_by_user_id INTEGER
        );
        CREATE TABLE IF NOT EXISTS task_assignees (
            task_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            PRIMARY KEY (task_id, user_id)
        );
        CREATE TABLE IF NOT EXISTS shift_plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            month TEXT NOT NULL,
            work_days INTEGER NOT NULL DEFAULT 22,
            daily_hours REAL NOT NULL DEFAULT 8,
            start_time TEXT NOT NULL DEFAULT '09:00',
            end_time TEXT NOT NULL DEFAULT '18:00',
            note TEXT NOT NULL DEFAULT '',
            weekdays TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS shift_attendance (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            date TEXT NOT NULL,
            hours REAL NOT NULL DEFAULT 0,
            approved INTEGER NOT NULL DEFAULT 0,
            approved_by INTEGER,
            note TEXT NOT NULL DEFAULT ''
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
        CREATE TABLE IF NOT EXISTS help_topics (
            key TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            body TEXT NOT NULL DEFAULT ''
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
            footer_text TEXT NOT NULL DEFAULT 'Bu tanıtım belgesi Gayrimenkul360 tarafından oluşturulmuştur.',
            val_title_min INTEGER NOT NULL DEFAULT 3,
            val_phone_digits INTEGER NOT NULL DEFAULT 10,
            val_email_required INTEGER NOT NULL DEFAULT 0,
            val_phone_required INTEGER NOT NULL DEFAULT 0,
            val_desc_min INTEGER NOT NULL DEFAULT 0,
            val_price_min REAL NOT NULL DEFAULT 0,
            val_area_min REAL NOT NULL DEFAULT 0,
            default_theme TEXT NOT NULL DEFAULT 'system',
            default_zoom REAL NOT NULL DEFAULT 1.0
        );
        "#,
    )?;

    // Eski veritabanlarında eksik kolonları ekle (göç)
    migrate(conn)?;

    // Varsayılan ayarlar
    conn.execute(
        "INSERT OR IGNORE INTO settings (id) VALUES (1)",
        [],
    )?;

    // Varsayılan ekran yardım içerikleri (idempotent)
    let seed: &[(&str, &str, &str)] = &[
        ("welcome", "Portföyler (Karşılama)", "En güncelden en eskiye portföy listesi.\n\n• Görsel/Liste görünümü üstteki düğmelerle değişir (varsayılan: Görsel).\n• İlçe, tür ve durum filtrelerini kullanın; arama kutusuna yazıp Enter'a basın.\n• Admin: 'Silinmişleri göster' ile mantıksal silinen kayıtları görürsünüz.\n• Kısayol: Ctrl + fare tekerleği ile kart boyutunu değiştirin."),
        ("portfolio_form", "Portföy Formu", "Tüm *'lı alanlar zorunludur.\n\n• İlçe ve mahalle ön tanımlı listeden seçilir.\n• Fiyata yazdığınız rakam binlik ayraçla otomatik biçimlenir; m² fiyatı otomatik hesaplanır.\n• Resimleri pencereye sürükleyip bırakabilir veya 'Resim Ekle' ile seçebilirsiniz; ilk resim kapak olur.\n• Konum: 'Haritada Seç' ile uydu görünümünde açıp koordinatı kopyalayıp yapıştırın; 'Mahalleyi Haritada Ara' ile Fethiye/mahalle araması yapılır.\n• Taşınmaz sahibi bilgisi CRM kaydı olarak saklanır (listesini yalnızca yönetici görür)."),
        ("portfolio_detail", "Portföy Detayı", "Tüm bilgiler, resim galerisi ve çıktı işlemleri.\n\n• Resme tıklayınca büyütülür.\n• PDF/DOCX: tüm resimler ızgara olarak belgeye eklenir.\n• WhatsApp ile paylaşım ve haritada açma butonları buradadır.\n• Silme mantıksaldır; yönetici geri yükleyebilir."),
        ("tasks", "Görevler", "Kişiye veya ekebe görev/not atarsınız.\n\n• Çoklu personel seçebilirsiniz; hiçbiri seçilmezse tüm ekibe gider.\n• Tarih alanı GG-AA-YYYY biçiminde otomatik tire ekler.\n• Atanan her kişiye bildirim düşer."),
        ("goals", "Hedefler", "Kişiye/ekibe dönemsel hedefler. Gerçekleşme portföy kayıtlarından otomatik hesaplanır. Dönemi GG-AA-YYYY olarak girin."),
        ("shifts", "Vardiya / Mesai", "3 sekme:\n\n• Vardiya Takvimi: haftanın günleri + saat penceresi atanır (aynı ay için birden fazla pencere eklenebilir). Herkes salt-okunur görür.\n• Mesai Onay Listesi: gerçekleşen gün/saat kaydı girilir, çoklu seçilip toplu onaylanır.\n• Vardiya Raporu: ay + personel seçilip PDF/CSV raporu alınır (devam yüzdesi dahil)."),
        ("crm", "CRM Kayıtları", "Taşınmaz sahibi/müşteri kayıtları. Yalnızca yönetici görür.\n\n• Düzenle ile bilgileri güncelleyin; Fotoğraf ile 256x256 kare kırpılmış profil fotoğrafı yükleyin.\n• Görsel/Liste görünümü ve silinmişleri gösterme üstteki düğmelerdedir."),
        ("users", "Kullanıcılar", "Çalışan hesapları ve rolleri (yönetici). Pasifleştirilen kullanıcı oturum açamaz."),
        ("settings", "Ayarlar Paneli", "Sekmeler:\n\n• Görünüm: tema (karanlık/aydınlık/sistem), renk teması (Mavi/Yeşil/Gri/Turuncu/Mor), yazı tipi ailesi ve punto, yakınlaştırma (Ctrl +/-/0).\n• Çıktı: PDF/DOCX belgelerinde görünecek alanlar ve yeni kullanıcılar için varsayılan tema.\n• Doğrulama: form kuralları (min karakter, min fiyat/alan, telefon/e-posta zorunluluğu).\n• Kullanıcılar: hesap ve rol yönetimi.\n• Yardım İçerikleri: bu ekrandaki açıklamaları düzenleyin.\n• Sistem Testi: tüm modülleri tek tuşla test edin."),
        ("profile", "Profilim", "Ad soyad, telefon, e-posta ve profil fotoğrafı. Bu bilgiler çıktı belgelerinin altında iletişim olarak görünür. Parolanızı buradan değiştirin."),
    ];
    for (key, title, body) in seed {
        conn.execute(
            "INSERT OR IGNORE INTO help_topics (key, title, body) VALUES (?1, ?2, ?3)",
            params![key, title, body],
        )?;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Göçler idempotent olmalı: init istedi kadar çağrılabilir, hata vermemeli.
    #[test]
    fn migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        init(&conn).unwrap();
        init(&conn).unwrap();
    }

    /// Bozuk göçle oluşmuş 'INTEGER' çöp kolonu temizlenmeli, gerçek kolonlar eklenmeli.
    #[test]
    fn cleans_junk_column_from_broken_migration() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT UNIQUE NOT NULL,
             password_hash TEXT NOT NULL, salt TEXT NOT NULL, full_name TEXT NOT NULL DEFAULT '',
             role TEXT NOT NULL DEFAULT 'agent', phone TEXT NOT NULL DEFAULT '',
             email TEXT NOT NULL DEFAULT '', photo TEXT, active INTEGER NOT NULL DEFAULT 1,
             created_at TEXT NOT NULL);
             CREATE TABLE settings (id INTEGER PRIMARY KEY CHECK (id = 1),
             header_text TEXT NOT NULL DEFAULT 'x', footer_text TEXT NOT NULL DEFAULT 'y');
             ALTER TABLE settings ADD COLUMN \"INTEGER\" INTEGER NOT NULL DEFAULT 3;",
        )
        .unwrap();

        init(&conn).unwrap();

        assert!(!column_exists(&conn, "settings", "INTEGER"), "çöp kolon temizlenmedi");
        assert!(column_exists(&conn, "settings", "val_title_min"), "val_title_min eklenmedi");
        assert!(column_exists(&conn, "settings", "default_theme"), "default_theme eklenmedi");
    }
}
