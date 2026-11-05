use std::io::Write;

use axum::extract::{Multipart, Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use rusqlite::{params, OptionalExtension};

use common::*;

use crate::auth::{bad_request, internal, not_found, unauthorized, AdminUser, CurrentUser};
use crate::db::{hash_password, new_salt, new_token, now};
use crate::state::{load_images, map_attendance, map_crm, map_portfolio, map_settings, map_shift_plan, map_task, map_user, AppState};

type ApiResult<T> = std::result::Result<T, Response>;

// =====================================================================
// AUTH
// =====================================================================

pub async fn login(State(state): State<AppState>, Json(req): Json<LoginRequest>) -> ApiResult<Json<AuthResponse>> {
    let conn = state.db.lock().unwrap();
    let row = conn
        .query_row(
            "SELECT id, password_hash, salt FROM users WHERE username = ?1 AND active = 1",
            [&req.username],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
        )
        .optional()
        .map_err(|e| internal(&e.to_string()))?;

    let (id, hash, salt) = match row {
        Some(v) => v,
        None => return Err(unauthorized("Kullanıcı adı veya parola hatalı")),
    };

    if hash_password(&req.password, &salt) != hash {
        return Err(unauthorized("Kullanıcı adı veya parola hatalı"));
    }

    let token = new_token();
    conn.execute(
        "INSERT INTO sessions (token, user_id, created_at) VALUES (?1, ?2, ?3)",
        params![token, id, now()],
    )
    .map_err(|e| internal(&e.to_string()))?;

    let user = conn
        .query_row("SELECT * FROM users WHERE id = ?1", [id], map_user)
        .map_err(|e| internal(&e.to_string()))?;

    Ok(Json(AuthResponse { token, user }))
}

pub async fn logout(State(state): State<AppState>, req: axum::http::HeaderMap) -> ApiResult<Json<serde_json::Value>> {
    if let Some(token) = req
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        let conn = state.db.lock().unwrap();
        let _ = conn.execute("DELETE FROM sessions WHERE token = ?1", [token]);
    }
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn me(CurrentUser(user): CurrentUser) -> Json<User> {
    Json(user)
}

pub async fn update_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<UpdateProfileRequest>,
) -> ApiResult<Json<User>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE users SET full_name = ?1, phone = ?2, email = ?3 WHERE id = ?4",
        params![req.full_name, req.phone, req.email, user.id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let user = conn
        .query_row("SELECT * FROM users WHERE id = ?1", [user.id], map_user)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(user))
}

pub async fn change_password(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<ChangePasswordRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let (hash, salt) = conn
        .query_row(
            "SELECT password_hash, salt FROM users WHERE id = ?1",
            [user.id],
            |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
        )
        .map_err(|e| internal(&e.to_string()))?;
    if hash_password(&req.old_password, &salt) != hash {
        return Err(bad_request("Mevcut parola hatalı"));
    }
    let nsalt = new_salt();
    let nhash = hash_password(&req.new_password, &nsalt);
    conn.execute(
        "UPDATE users SET password_hash = ?1, salt = ?2 WHERE id = ?3",
        params![nhash, nsalt, user.id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn upload_my_photo(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    mut multipart: Multipart,
) -> ApiResult<Json<User>> {
    let filename = save_first_upload(&state, &mut multipart).await?;
    let conn = state.db.lock().unwrap();
    conn.execute("UPDATE users SET photo = ?1 WHERE id = ?2", params![filename, user.id])
        .map_err(|e| internal(&e.to_string()))?;
    let user = conn
        .query_row("SELECT * FROM users WHERE id = ?1", [user.id], map_user)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(user))
}

// =====================================================================
// USERS
// =====================================================================

pub async fn list_users(State(state): State<AppState>, _u: CurrentUser) -> ApiResult<Json<Vec<User>>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT * FROM users ORDER BY full_name COLLATE NOCASE").map_err(|e| internal(&e.to_string()))?;
    let users = stmt
        .query_map([], map_user)
        .map_err(|e| internal(&e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(Json(users))
}

pub async fn create_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<Json<User>> {
    let conn = state.db.lock().unwrap();
    let salt = new_salt();
    let hash = hash_password(&req.password, &salt);
    conn.execute(
        "INSERT INTO users (username, password_hash, salt, full_name, role, phone, email, active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
        params![req.username, hash, salt, req.full_name, req.role.as_str(), req.phone, req.email, now()],
    )
    .map_err(|e| bad_request(&format!("Kullanıcı oluşturulamadı: {e}")))?;
    let id = conn.last_insert_rowid();
    let user = conn
        .query_row("SELECT * FROM users WHERE id = ?1", [id], map_user)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(user))
}

pub async fn update_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i64>,
    Json(req): Json<CreateUserRequest>,
) -> ApiResult<Json<User>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE users SET full_name = ?1, role = ?2, phone = ?3, email = ?4 WHERE id = ?5",
        params![req.full_name, req.role.as_str(), req.phone, req.email, id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    if !req.password.is_empty() {
        let salt = new_salt();
        let hash = hash_password(&req.password, &salt);
        conn.execute("UPDATE users SET password_hash = ?1, salt = ?2 WHERE id = ?3", params![hash, salt, id])
            .map_err(|e| internal(&e.to_string()))?;
    }
    let user = conn
        .query_row("SELECT * FROM users WHERE id = ?1", [id], map_user)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(user))
}

pub async fn toggle_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<User>> {
    let conn = state.db.lock().unwrap();
    conn.execute("UPDATE users SET active = CASE active WHEN 1 THEN 0 ELSE 1 END WHERE id = ?1", [id])
        .map_err(|e| internal(&e.to_string()))?;
    // pasifleşen kullanıcının oturumlarını temizle
    conn.execute("DELETE FROM sessions WHERE user_id = ?1 AND (SELECT active FROM users WHERE id = ?1) = 0", [id])
        .map_err(|e| internal(&e.to_string()))?;
    let user = conn
        .query_row("SELECT * FROM users WHERE id = ?1", [id], map_user)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(user))
}

// =====================================================================
// PORTFOLYO
// =====================================================================

#[derive(serde::Deserialize, Default)]
pub struct PortfolioQuery {
    pub q: Option<String>,
    pub ilce: Option<String>,
    pub mahalle: Option<String>,
    pub listing_type: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    pub owner: Option<i64>,
    pub show_deleted: Option<i64>,
}

pub async fn list_portfolios(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Query(query): Query<PortfolioQuery>,
) -> ApiResult<Json<Vec<Portfolio>>> {
    let conn = state.db.lock().unwrap();
    let mut sql = String::from(
        "SELECT p.*, u.full_name AS created_by_name
         FROM portfolios p LEFT JOIN users u ON u.id = p.created_by WHERE 1=1",
    );
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    // Mantıksal silme: silinenler yalnızca admin + açık istekle görünür
    let show_deleted = query.show_deleted.unwrap_or(0) == 1 && user.role == Role::Admin;
    if !show_deleted {
        sql.push_str(" AND p.is_deleted = 0");
    }

    if let Some(q) = query.q.filter(|s| !s.trim().is_empty()) {
        sql.push_str(" AND (p.title LIKE ?1 OR p.description LIKE ?1 OR p.address LIKE ?1)");
        args.push(Box::new(format!("%{}%", q)));
    }
    let add = |sql: &mut String, args: &mut Vec<Box<dyn rusqlite::ToSql>>, col: &str, val: Option<String>| {
        if let Some(v) = val.filter(|s| !s.trim().is_empty()) {
            let idx = args.len() + 1;
            sql.push_str(&format!(" AND p.{col} = ?{idx}"));
            args.push(Box::new(v));
        }
    };
    add(&mut sql, &mut args, "ilce", query.ilce.clone());
    add(&mut sql, &mut args, "mahalle", query.mahalle.clone());
    add(&mut sql, &mut args, "listing_type", query.listing_type.clone());
    add(&mut sql, &mut args, "category", query.category.clone());
    add(&mut sql, &mut args, "status", query.status.clone());
    if let Some(owner) = query.owner {
        let idx = args.len() + 1;
        sql.push_str(&format!(" AND p.created_by = ?{idx}"));
        args.push(Box::new(owner));
    }
    sql.push_str(" ORDER BY p.created_at DESC, p.id DESC");

    let borrowed: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql).map_err(|e| internal(&e.to_string()))?;
    let mut list: Vec<Portfolio> = stmt
        .query_map(borrowed.as_slice(), map_portfolio)
        .map_err(|e| internal(&e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    for p in list.iter_mut() {
        p.images = load_images(&conn, p.id);
    }
    Ok(Json(list))
}

pub async fn get_portfolio(
    State(state): State<AppState>,
    _u: CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<Portfolio>> {
    let conn = state.db.lock().unwrap();
    let mut p = conn
        .query_row(
            "SELECT p.*, u.full_name AS created_by_name FROM portfolios p
             LEFT JOIN users u ON u.id = p.created_by WHERE p.id = ?1",
            [id],
            map_portfolio,
        )
        .map_err(|_| not_found("Portföy bulunamadı"))?;
    p.images = load_images(&conn, p.id);
    Ok(Json(p))
}

/// Ayarlarda tanımlı doğrulama kurallarını portföy girdisine uygular.
fn validate_portfolio_input(conn: &rusqlite::Connection, input: &PortfolioInput) -> ApiResult<()> {
    let rules = conn
        .query_row("SELECT * FROM settings WHERE id = 1", [], map_settings)
        .unwrap_or_default();

    if (input.title.trim().chars().count() as i64) < rules.val_title_min {
        return Err(bad_request(&format!(
            "Başlık en az {} karakter olmalıdır",
            rules.val_title_min
        )));
    }
    if rules.val_desc_min > 0 && (input.description.trim().chars().count() as i64) < rules.val_desc_min {
        return Err(bad_request(&format!(
            "Açıklama en az {} karakter olmalıdır",
            rules.val_desc_min
        )));
    }
    if input.price < rules.val_price_min {
        return Err(bad_request(&format!(
            "Fiyat en az {} olmalıdır",
            format_num(rules.val_price_min)
        )));
    }
    if input.area_m2 < rules.val_area_min {
        return Err(bad_request(&format!(
            "Alan en az {} m² olmalıdır",
            format_num(rules.val_area_min)
        )));
    }
    Ok(())
}

fn format_num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn insert_portfolio(conn: &rusqlite::Connection, user: &User, input: &PortfolioInput, id: Option<i64>) -> ApiResult<i64> {
    let ts = now();
    if let Some(pid) = id {
        conn.execute(
            "UPDATE portfolios SET title=?1, listing_type=?2, category=?3, status=?4, price=?5, currency=?6,
             area_m2=?7, room_count=?8, building_age=?9, floor=?10, total_floors=?11, heating=?12, furnished=?13,
             il=?14, ilce=?15, mahalle=?16, address=?17, latitude=?18, longitude=?19, description=?20, owner_crm_id=?21,
             updated_at=?22 WHERE id=?23",
            params![
                input.title, input.listing_type, input.category, input.status, input.price, input.currency,
                input.area_m2, input.room_count, input.building_age, input.floor, input.total_floors, input.heating,
                input.furnished as i64, input.il, input.ilce, input.mahalle, input.address, input.latitude,
                input.longitude, input.description, input.owner_crm_id, ts, pid
            ],
        )
        .map_err(|e| internal(&e.to_string()))?;
        Ok(pid)
    } else {
        conn.execute(
            "INSERT INTO portfolios (title, listing_type, category, status, price, currency, area_m2, room_count,
             building_age, floor, total_floors, heating, furnished, il, ilce, mahalle, address, latitude, longitude,
             description, owner_crm_id, created_by, created_at, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?23)",
            params![
                input.title, input.listing_type, input.category, input.status, input.price, input.currency,
                input.area_m2, input.room_count, input.building_age, input.floor, input.total_floors, input.heating,
                input.furnished as i64, input.il, input.ilce, input.mahalle, input.address, input.latitude,
                input.longitude, input.description, input.owner_crm_id, user.id, ts
            ],
        )
        .map_err(|e| internal(&e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }
}

pub async fn create_portfolio(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(input): Json<PortfolioInput>,
) -> ApiResult<Json<Portfolio>> {
    if input.title.trim().is_empty() {
        return Err(bad_request("Başlık zorunludur"));
    }
    let conn = state.db.lock().unwrap();
    validate_portfolio_input(&conn, &input)?;
    let id = insert_portfolio(&conn, &user, &input, None)?;
    let p = conn
        .query_row(
            "SELECT p.*, u.full_name AS created_by_name FROM portfolios p
             LEFT JOIN users u ON u.id = p.created_by WHERE p.id = ?1",
            [id],
            map_portfolio,
        )
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(p))
}

pub async fn update_portfolio(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
    Json(input): Json<PortfolioInput>,
) -> ApiResult<Json<Portfolio>> {
    let conn = state.db.lock().unwrap();
    let owner: i64 = conn
        .query_row("SELECT created_by FROM portfolios WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| not_found("Portföy bulunamadı"))?;
    if owner != user.id && user.role != Role::Admin {
        return Err(unauthorized("Bu portföyü düzenleme yetkiniz yok"));
    }
    validate_portfolio_input(&conn, &input)?;
    insert_portfolio(&conn, &user, &input, Some(id))?;
    let mut p = conn
        .query_row(
            "SELECT p.*, u.full_name AS created_by_name FROM portfolios p
             LEFT JOIN users u ON u.id = p.created_by WHERE p.id = ?1",
            [id],
            map_portfolio,
        )
        .map_err(|e| internal(&e.to_string()))?;
    p.images = load_images(&conn, p.id);
    Ok(Json(p))
}

/// Mantıksal silme (soft-delete): kayıt gizlenir, veritabanında kalır.
/// Silme bildirimi tüm yöneticilere düşer (silen kullanıcı hariç).
fn notify_admins_of_delete(conn: &rusqlite::Connection, actor: &User, kind: &str, subject: &str) {
    let admins: Vec<i64> = {
        let mut stmt = conn.prepare("SELECT id FROM users WHERE role='admin' AND active=1").unwrap();
        stmt.query_map([], |r| r.get(0)).map(|it| it.filter_map(|r| r.ok()).collect()).unwrap_or_default()
    };
    let title = format!("{subject} silme komutu aldı");
    let body = format!(
        "{} kaydı {} ({}) kullanıcısı tarafından silindi. Yönetici panelinden geri yüklenebilir.",
        kind, actor.full_name, actor.username
    );
    for admin_id in admins {
        if admin_id == actor.id {
            continue;
        }
        conn.execute(
            "INSERT INTO notifications (user_id, title, body, kind, read, created_at)
             VALUES (?1, ?2, ?3, 'delete', 0, ?4)",
            params![admin_id, title, body, now()],
        )
        .ok();
    }
}

pub async fn delete_portfolio(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let owner: i64 = conn
        .query_row("SELECT created_by FROM portfolios WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| not_found("Portföy bulunamadı"))?;
    if owner != user.id && user.role != Role::Admin {
        return Err(unauthorized("Bu portföyü silme yetkiniz yok"));
    }
    conn.execute(
        "UPDATE portfolios SET is_deleted = 1, deleted_by_user_id = ?1 WHERE id = ?2",
        params![user.id, id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    notify_admins_of_delete(&conn, &user, "Portföy", "Portföy");
    Ok(Json(serde_json::json!({ "ok": true, "soft_deleted": true })))
}

pub async fn restore_portfolio(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE portfolios SET is_deleted = 0, deleted_by_user_id = NULL WHERE id = ?1",
        [id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn upload_portfolio_images(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> ApiResult<Json<Vec<String>>> {
    {
        let conn = state.db.lock().unwrap();
        let owner: i64 = conn
            .query_row("SELECT created_by FROM portfolios WHERE id = ?1", [id], |r| r.get(0))
            .map_err(|_| not_found("Portföy bulunamadı"))?;
        if owner != user.id && user.role != Role::Admin {
            return Err(unauthorized("Bu portföye resim ekleme yetkiniz yok"));
        }
    }

    let mut saved = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(|e| bad_request(&e.to_string()))? {
        let orig = field.file_name().unwrap_or("resim.jpg").to_string();
        let ext = std::path::Path::new(&orig)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg")
            .to_lowercase();
        let data = field.bytes().await.map_err(|e| bad_request(&e.to_string()))?;
        if data.is_empty() {
            continue;
        }
        let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
        let path = state.upload_dir.join(&filename);
        let mut f = std::fs::File::create(&path).map_err(|e| internal(&e.to_string()))?;
        f.write_all(&data).map_err(|e| internal(&e.to_string()))?;

        let conn = state.db.lock().unwrap();
        let order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM portfolio_images WHERE portfolio_id = ?1",
                [id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        conn.execute(
            "INSERT INTO portfolio_images (portfolio_id, filename, sort_order) VALUES (?1, ?2, ?3)",
            params![id, filename, order],
        )
        .map_err(|e| internal(&e.to_string()))?;
        saved.push(filename);
    }
    Ok(Json(saved))
}

pub async fn delete_portfolio_image(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path((id, filename)): Path<(i64, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let owner: i64 = conn
        .query_row("SELECT created_by FROM portfolios WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| not_found("Portföy bulunamadı"))?;
    if owner != user.id && user.role != Role::Admin {
        return Err(unauthorized("Yetkisiz işlem"));
    }
    conn.execute(
        "DELETE FROM portfolio_images WHERE portfolio_id = ?1 AND filename = ?2",
        params![id, filename],
    )
    .map_err(|e| internal(&e.to_string()))?;
    drop(conn);
    let _ = std::fs::remove_file(state.upload_dir.join(&filename));
    Ok(Json(serde_json::json!({ "ok": true })))
}

// =====================================================================
// CRM  (liste/okuma/güncelleme/silme sadece admin)
// =====================================================================

#[derive(serde::Deserialize, Default)]
pub struct CrmQuery {
    pub show_deleted: Option<i64>,
}

pub async fn list_crm(
    State(state): State<AppState>,
    _admin: AdminUser,
    Query(q): Query<CrmQuery>,
) -> ApiResult<Json<Vec<CrmRecord>>> {
    let conn = state.db.lock().unwrap();
    let show_deleted = q.show_deleted.unwrap_or(0) == 1;
    let sql = if show_deleted {
        "SELECT c.*, u.full_name AS created_by_name FROM crm c
         LEFT JOIN users u ON u.id = c.created_by ORDER BY c.is_deleted, c.created_at DESC, c.id DESC"
    } else {
        "SELECT c.*, u.full_name AS created_by_name FROM crm c
         LEFT JOIN users u ON u.id = c.created_by WHERE c.is_deleted = 0
         ORDER BY c.created_at DESC, c.id DESC"
    };
    let mut stmt = conn.prepare(sql).map_err(|e| internal(&e.to_string()))?;
    let list = stmt
        .query_map([], map_crm)
        .map_err(|e| internal(&e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(Json(list))
}

pub async fn get_crm(State(state): State<AppState>, _admin: AdminUser, Path(id): Path<i64>) -> ApiResult<Json<CrmRecord>> {
    let conn = state.db.lock().unwrap();
    let rec = conn
        .query_row(
            "SELECT c.*, u.full_name AS created_by_name FROM crm c
             LEFT JOIN users u ON u.id = c.created_by WHERE c.id = ?1",
            [id],
            map_crm,
        )
        .map_err(|_| not_found("Kayıt bulunamadı"))?;
    Ok(Json(rec))
}

/// Herhangi bir kullanıcı (danışman) portföy formundan yeni CRM kaydı oluşturabilir.
pub async fn create_crm(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(input): Json<CrmInput>,
) -> ApiResult<Json<CrmRecord>> {
    if input.full_name.trim().is_empty() {
        return Err(bad_request("Ad Soyad zorunludur"));
    }
    let conn = state.db.lock().unwrap();
    conn.execute(
        "INSERT INTO crm (full_name, phone, email, address, owner_type, notes, created_by, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![input.full_name, input.phone, input.email, input.address, input.owner_type, input.notes, user.id, now()],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let id = conn.last_insert_rowid();
    let rec = conn
        .query_row(
            "SELECT c.*, u.full_name AS created_by_name FROM crm c
             LEFT JOIN users u ON u.id = c.created_by WHERE c.id = ?1",
            [id],
            map_crm,
        )
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(rec))
}

pub async fn update_crm(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i64>,
    Json(input): Json<CrmInput>,
) -> ApiResult<Json<CrmRecord>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE crm SET full_name=?1, phone=?2, email=?3, address=?4, owner_type=?5, notes=?6 WHERE id=?7",
        params![input.full_name, input.phone, input.email, input.address, input.owner_type, input.notes, id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let rec = conn
        .query_row(
            "SELECT c.*, u.full_name AS created_by_name FROM crm c
             LEFT JOIN users u ON u.id = c.created_by WHERE c.id = ?1",
            [id],
            map_crm,
        )
        .map_err(|_| not_found("Kayıt bulunamadı"))?;
    Ok(Json(rec))
}

pub async fn delete_crm(State(state): State<AppState>, CurrentUser(user): CurrentUser, Path(id): Path<i64>) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE crm SET is_deleted = 1, deleted_by_user_id = ?1 WHERE id = ?2",
        params![user.id, id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    notify_admins_of_delete(&conn, &user, "CRM", "CRM kaydı");
    Ok(Json(serde_json::json!({ "ok": true, "soft_deleted": true })))
}

pub async fn restore_crm(State(state): State<AppState>, _admin: AdminUser, Path(id): Path<i64>) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE crm SET is_deleted = 0, deleted_by_user_id = NULL WHERE id = ?1",
        [id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// CRM (müşteri/sahip) profil fotoğrafı yükler; kare kırpma + 256px standardizasyon.
pub async fn upload_crm_photo(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> ApiResult<Json<CrmRecord>> {
    let filename = {
        let mut saved: Option<String> = None;
        while let Some(field) = multipart.next_field().await.map_err(|e| bad_request(&e.to_string()))? {
            let data = field.bytes().await.map_err(|e| bad_request(&e.to_string()))?;
            if data.is_empty() {
                continue;
            }
            let img = image::load_from_memory(&data)
                .map_err(|_| bad_request("Geçerli bir resim dosyası değil"))?;
            // Merkezden kare kırpma
            let (w, h) = (img.width(), img.height());
            let side = w.min(h);
            let x = (w - side) / 2;
            let y = (h - side) / 2;
            let cropped = img.crop_imm(x, y, side, side);
            let resized = cropped.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
            let filename = format!("{}.jpg", uuid::Uuid::new_v4());
            let path = state.upload_dir.join(&filename);
            let mut cursor = std::io::Cursor::new(Vec::new());
            resized
                .write_to(&mut cursor, image::ImageFormat::Jpeg)
                .map_err(|e| internal(&e.to_string()))?;
            std::fs::write(&path, cursor.into_inner()).map_err(|e| internal(&e.to_string()))?;
            saved = Some(filename);
            break;
        }
        match saved {
            Some(f) => f,
            None => return Err(bad_request("Dosya bulunamadı")),
        }
    };
    let conn = state.db.lock().unwrap();
    conn.execute("UPDATE crm SET photo = ?1 WHERE id = ?2", params![filename, id])
        .map_err(|e| internal(&e.to_string()))?;
    let rec = conn
        .query_row(
            "SELECT c.*, u.full_name AS created_by_name FROM crm c
             LEFT JOIN users u ON u.id = c.created_by WHERE c.id = ?1",
            [id],
            map_crm,
        )
        .map_err(|_| not_found("Kayıt bulunamadı"))?;
    Ok(Json(rec))
}

// =====================================================================
// GOREVLER
// =====================================================================

pub async fn list_tasks(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Vec<Task>>> {
    let conn = state.db.lock().unwrap();
    let sql = "SELECT t.*, NULL AS assigned_to_name, b.full_name AS assigned_by_name
               FROM tasks t
               LEFT JOIN users b ON b.id = t.assigned_by
               WHERE t.is_deleted = 0 AND (?1 = 1 OR t.assigned_to IS NULL OR t.assigned_by = ?2
                     OR EXISTS (SELECT 1 FROM task_assignees ta WHERE ta.task_id = t.id AND ta.user_id = ?2))
               ORDER BY t.done, t.due_at, t.id DESC";
    let is_admin = (user.role == Role::Admin) as i64;
    let mut stmt = conn.prepare(sql).map_err(|e| internal(&e.to_string()))?;
    let mut list: Vec<Task> = stmt
        .query_map(params![is_admin, user.id], map_task)
        .map_err(|e| internal(&e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    // Çoklu atama bilgilerini doldur
    for t in list.iter_mut() {
        let mut ids_stmt = conn
            .prepare(
                "SELECT ta.user_id, COALESCE(u.full_name, u.username) FROM task_assignees ta
                 LEFT JOIN users u ON u.id = ta.user_id WHERE ta.task_id = ?1",
            )
            .map_err(|e| internal(&e.to_string()))?;
        let rows = ids_stmt
            .query_map([t.id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| internal(&e.to_string()))?;
        let pairs: Vec<(i64, String)> = rows.filter_map(|r| r.ok()).collect();
        t.assigned_to_ids = pairs.iter().map(|(id, _)| *id).collect();
        if !pairs.is_empty() {
            t.assigned_to_name = pairs.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join(", ");
            t.assigned_to = Some(t.assigned_to_ids[0]);
        }
    }
    Ok(Json(list))
}

pub async fn create_task(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(input): Json<TaskInput>,
) -> ApiResult<Json<Task>> {
    if input.title.trim().is_empty() {
        return Err(bad_request("Görev başlığı zorunludur"));
    }
    let conn = state.db.lock().unwrap();
    let first = input.assigned_to.first().copied();
    conn.execute(
        "INSERT INTO tasks (title, description, assigned_to, assigned_by, due_at, priority, done, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,0,?7)",
        params![input.title, input.description, first, user.id, input.due_at, input.priority, now()],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let id = conn.last_insert_rowid();

    // Çoklu atama + her atanan kişiye bildirim
    for assignee in &input.assigned_to {
        conn.execute(
            "INSERT OR IGNORE INTO task_assignees (task_id, user_id) VALUES (?1, ?2)",
            params![id, assignee],
        )
        .map_err(|e| internal(&e.to_string()))?;
        if *assignee != user.id {
            let body = if input.due_at.trim().is_empty() {
                format!("{} tarafından yeni görev atandı.", user.full_name)
            } else {
                format!("{} tarafından atandı. Zaman: {}", user.full_name, input.due_at)
            };
            conn.execute(
                "INSERT INTO notifications (user_id, title, body, kind, read, created_at)
                 VALUES (?1, ?2, ?3, 'task', 0, ?4)",
                params![assignee, input.title, body, now()],
            )
            .ok();
        }
    }

    let mut task = conn
        .query_row(
            "SELECT t.*, NULL AS assigned_to_name, b.full_name AS assigned_by_name FROM tasks t
             LEFT JOIN users b ON b.id = t.assigned_by WHERE t.id = ?1",
            [id],
            map_task,
        )
        .map_err(|e| internal(&e.to_string()))?;
    fill_assignees(&conn, &mut task);
    Ok(Json(task))
}

/// Görevin çoklu atama kimlerini ve adlarını doldurur.
fn fill_assignees(conn: &rusqlite::Connection, t: &mut Task) {
    let mut ids_stmt = conn
        .prepare(
            "SELECT ta.user_id, COALESCE(u.full_name, u.username) FROM task_assignees ta
             LEFT JOIN users u ON u.id = ta.user_id WHERE ta.task_id = ?1",
        )
        .ok();
    if let Some(stmt) = ids_stmt.as_mut() {
        if let Ok(rows) = stmt.query_map([t.id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))) {
            let pairs: Vec<(i64, String)> = rows.filter_map(|r| r.ok()).collect();
            if !pairs.is_empty() {
                t.assigned_to_ids = pairs.iter().map(|(id, _)| *id).collect();
                t.assigned_to_name = pairs.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>().join(", ");
                t.assigned_to = Some(t.assigned_to_ids[0]);
            }
        }
    }
}

pub async fn toggle_task(State(state): State<AppState>, _u: CurrentUser, Path(id): Path<i64>) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute("UPDATE tasks SET done = CASE done WHEN 1 THEN 0 ELSE 1 END WHERE id = ?1", [id])
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn delete_task(State(state): State<AppState>, CurrentUser(user): CurrentUser, Path(id): Path<i64>) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    let by: i64 = conn
        .query_row("SELECT assigned_by FROM tasks WHERE id = ?1", [id], |r| r.get(0))
        .map_err(|_| not_found("Görev bulunamadı"))?;
    if by != user.id && user.role != Role::Admin {
        return Err(unauthorized("Bu görevi silme yetkiniz yok"));
    }
    conn.execute(
        "UPDATE tasks SET is_deleted = 1, deleted_by_user_id = ?1 WHERE id = ?2",
        params![user.id, id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    notify_admins_of_delete(&conn, &user, "Görev", "Görev");
    Ok(Json(serde_json::json!({ "ok": true, "soft_deleted": true })))
}

// =====================================================================
// HEDEFLER
// =====================================================================

pub async fn list_goals(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Vec<Goal>>> {
    let conn = state.db.lock().unwrap();
    let sql = "SELECT g.*, u.full_name AS user_name FROM goals g
               LEFT JOIN users u ON u.id = g.user_id
               WHERE ?1 = 1 OR g.scope = 'team' OR g.user_id = ?2
               ORDER BY g.id DESC";
    let is_admin = (user.role == Role::Admin) as i64;
    let mut stmt = conn.prepare(sql).map_err(|e| internal(&e.to_string()))?;
    let rows = stmt
        .query_map(params![is_admin, user.id], |row| {
            Ok(Goal {
                id: row.get("id")?,
                scope: row.get("scope")?,
                user_id: row.get("user_id")?,
                user_name: row.get::<_, Option<String>>("user_name")?.unwrap_or_default(),
                title: row.get("title")?,
                metric: row.get("metric")?,
                target_count: row.get("target_count")?,
                current_count: 0,
                period_start: row.get("period_start")?,
                period_end: row.get("period_end")?,
                created_at: row.get("created_at")?,
            })
        })
        .map_err(|e| internal(&e.to_string()))?;

    let mut goals: Vec<Goal> = rows.filter_map(|r| r.ok()).collect();
    // İlerleme hesabı
    for g in goals.iter_mut() {
        g.current_count = compute_goal_progress(&conn, g);
    }
    Ok(Json(goals))
}

/// Hedefe göre gerçekleşen sayıyı portföyler tablosundan hesaplar.
fn compute_goal_progress(conn: &rusqlite::Connection, g: &Goal) -> i64 {
    let mut where_user = String::new();
    let mut args: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if g.scope == "user" {
        if let Some(uid) = g.user_id {
            args.push(Box::new(uid));
            where_user = format!(" AND created_by = ?{}", args.len());
        }
    }
    let mut period = String::new();
    if !g.period_start.trim().is_empty() {
        args.push(Box::new(g.period_start.clone()));
        period.push_str(&format!(" AND created_at >= ?{}", args.len()));
    }
    if !g.period_end.trim().is_empty() {
        args.push(Box::new(format!("{} 23:59:59", g.period_end)));
        period.push_str(&format!(" AND created_at <= ?{}", args.len()));
    }

    let metric = g.metric.to_lowercase();
    let cond = if metric.contains("yayin") || metric.contains("yayın") {
        " AND status = 'yayinda'"
    } else if metric.contains("kiralama") || metric.contains("kiral") {
        " AND listing_type = 'kiralik' AND status = 'kiralandi'"
    } else if metric.contains("sat") {
        " AND status = 'satildi'"
    } else {
        ""
    };
    let sql = format!("SELECT COUNT(*) FROM portfolios WHERE 1=1{where_user}{period}{cond}");
    let borrowed: Vec<&dyn rusqlite::ToSql> = args.iter().map(|b| b.as_ref()).collect();
    conn.query_row(&sql, borrowed.as_slice(), |r| r.get(0)).unwrap_or(0)
}

pub async fn create_goal(State(state): State<AppState>, _admin: AdminUser, Json(input): Json<GoalInput>) -> ApiResult<Json<Goal>> {
    if input.title.trim().is_empty() {
        return Err(bad_request("Hedef başlığı zorunludur"));
    }
    let conn = state.db.lock().unwrap();
    conn.execute(
        "INSERT INTO goals (scope, user_id, title, metric, target_count, period_start, period_end, created_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
        params![input.scope, input.user_id, input.title, input.metric, input.target_count, input.period_start, input.period_end, now()],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let id = conn.last_insert_rowid();

    // Hedef atanan kullanıcıya bildirim
    if let Some(uid) = input.user_id {
        conn.execute(
            "INSERT INTO notifications (user_id, title, body, kind, read, created_at)
             VALUES (?1, ?2, ?3, 'goal', 0, ?4)",
            params![uid, "Yeni hedef atandı", input.title, now()],
        )
        .ok();
    }

    let g = conn
        .query_row(
            "SELECT g.*, u.full_name AS user_name FROM goals g LEFT JOIN users u ON u.id = g.user_id WHERE g.id = ?1",
            [id],
            |row| {
                Ok(Goal {
                    id: row.get("id")?,
                    scope: row.get("scope")?,
                    user_id: row.get("user_id")?,
                    user_name: row.get::<_, Option<String>>("user_name")?.unwrap_or_default(),
                    title: row.get("title")?,
                    metric: row.get("metric")?,
                    target_count: row.get("target_count")?,
                    current_count: 0,
                    period_start: row.get("period_start")?,
                    period_end: row.get("period_end")?,
                    created_at: row.get("created_at")?,
                })
            },
        )
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(g))
}

pub async fn delete_goal(State(state): State<AppState>, _admin: AdminUser, Path(id): Path<i64>) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute("DELETE FROM goals WHERE id = ?1", [id]).map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// =====================================================================
// VARDİYA PLANI VE MESAİ ONAYI
// =====================================================================

/// Tüm vardiya planları (salt-okunur; kullanıcılar ekip planını görebilir).
pub async fn list_shift_plans(State(state): State<AppState>, _u: CurrentUser) -> ApiResult<Json<Vec<ShiftPlan>>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT s.*, u.full_name AS user_name FROM shift_plans s
             LEFT JOIN users u ON u.id = s.user_id
             ORDER BY s.month DESC, u.full_name COLLATE NOCASE",
        )
        .map_err(|e| internal(&e.to_string()))?;
    let list = stmt
        .query_map([], map_shift_plan)
        .map_err(|e| internal(&e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(Json(list))
}

/// Yönetici personel için aylık/haftalık vardiya planı atar (aynı ay için günceller).
pub async fn upsert_shift_plan(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(input): Json<ShiftPlanInput>,
) -> ApiResult<Json<ShiftPlan>> {
    if input.month.trim().is_empty() {
        return Err(bad_request("Ay zorunludur (örn. 2026-11)"));
    }
    let conn = state.db.lock().unwrap();
    conn.execute(
        "DELETE FROM shift_plans WHERE user_id = ?1 AND month = ?2",
        params![input.user_id, input.month],
    )
    .map_err(|e| internal(&e.to_string()))?;
    conn.execute(
        "INSERT INTO shift_plans (user_id, month, work_days, daily_hours, start_time, end_time, note)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![input.user_id, input.month, input.work_days, input.daily_hours, input.start_time, input.end_time, input.note],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let id = conn.last_insert_rowid();
    let plan = conn
        .query_row(
            "SELECT s.*, u.full_name AS user_name FROM shift_plans s
             LEFT JOIN users u ON u.id = s.user_id WHERE s.id = ?1",
            [id],
            map_shift_plan,
        )
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(plan))
}

/// Mesai kayıtları: danışman yalnızca kendi kayıtlarını, admin tümünü görür.
pub async fn list_attendance(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Vec<ShiftAttendance>>> {
    let conn = state.db.lock().unwrap();
    let is_admin = user.role == Role::Admin;
    let sql = if is_admin {
        "SELECT a.*, u.full_name AS user_name FROM shift_attendance a
         LEFT JOIN users u ON u.id = a.user_id ORDER BY a.date DESC, a.id DESC LIMIT 500"
    } else {
        "SELECT a.*, u.full_name AS user_name FROM shift_attendance a
         LEFT JOIN users u ON u.id = a.user_id WHERE a.user_id = ?1
         ORDER BY a.date DESC, a.id DESC LIMIT 500"
    };
    let mut stmt = conn.prepare(sql).map_err(|e| internal(&e.to_string()))?;
    let list = if is_admin {
        stmt.query_map([], map_attendance)
    } else {
        stmt.query_map([user.id], map_attendance)
    }
    .map_err(|e| internal(&e.to_string()))?
    .filter_map(|r| r.ok())
    .collect();
    Ok(Json(list))
}

/// Yönetici gerçekleşen mesai kaydı girer (gün + saat).
pub async fn upsert_attendance(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(input): Json<ShiftAttendanceInput>,
) -> ApiResult<Json<ShiftAttendance>> {
    if input.date.trim().is_empty() {
        return Err(bad_request("Tarih zorunludur (YYYY-MM-DD)"));
    }
    let conn = state.db.lock().unwrap();
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM shift_attendance WHERE user_id = ?1 AND date = ?2",
            params![input.user_id, input.date],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| internal(&e.to_string()))?;
    let id = match existing {
        Some(id) => {
            conn.execute(
                "UPDATE shift_attendance SET hours = ?1, note = ?2 WHERE id = ?3",
                params![input.hours, input.note, id],
            )
            .map_err(|e| internal(&e.to_string()))?;
            id
        }
        None => {
            conn.execute(
                "INSERT INTO shift_attendance (user_id, date, hours, approved, note) VALUES (?1,?2,?3,0,?4)",
                params![input.user_id, input.date, input.hours, input.note],
            )
            .map_err(|e| internal(&e.to_string()))?;
            conn.last_insert_rowid()
        }
    };
    let rec = conn
        .query_row(
            "SELECT a.*, u.full_name AS user_name FROM shift_attendance a
             LEFT JOIN users u ON u.id = a.user_id WHERE a.id = ?1",
            [id],
            map_attendance,
        )
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(rec))
}

/// Yönetici mesai kaydını onaylar / onayı kaldırır.
pub async fn toggle_attendance(
    State(state): State<AppState>,
    admin: AdminUser,
    Path(id): Path<i64>,
) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE shift_attendance
         SET approved = CASE approved WHEN 1 THEN 0 ELSE 1 END,
             approved_by = CASE approved WHEN 1 THEN NULL ELSE ?1 END
         WHERE id = ?2",
        params![admin.0.id, id],
    )
    .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// =====================================================================
// BILDIRIMLER
// =====================================================================

pub async fn list_notifications(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Vec<Notification>>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT * FROM notifications WHERE user_id = ?1 ORDER BY read, id DESC LIMIT 100")
        .map_err(|e| internal(&e.to_string()))?;
    let list = stmt
        .query_map([user.id], |row| {
            Ok(Notification {
                id: row.get("id")?,
                user_id: row.get("user_id")?,
                title: row.get("title")?,
                body: row.get("body")?,
                kind: row.get("kind")?,
                read: row.get::<_, i64>("read")? != 0,
                created_at: row.get("created_at")?,
            })
        })
        .map_err(|e| internal(&e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(Json(list))
}

pub async fn read_all_notifications(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<serde_json::Value>> {
    let conn = state.db.lock().unwrap();
    conn.execute("UPDATE notifications SET read = 1 WHERE user_id = ?1", [user.id])
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// =====================================================================
// AYARLAR
// =====================================================================

pub async fn get_settings(State(state): State<AppState>, _u: CurrentUser) -> ApiResult<Json<OutputSettings>> {
    let conn = state.db.lock().unwrap();
    let s = conn
        .query_row("SELECT * FROM settings WHERE id = 1", [], map_settings)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(s))
}

pub async fn update_settings(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(s): Json<OutputSettings>,
) -> ApiResult<Json<OutputSettings>> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE settings SET show_name=?1, show_phone=?2, show_email=?3, show_price=?4, show_address=?5,
         show_description=?6, show_cover_image=?7, header_text=?8, footer_text=?9,
         val_title_min=?10, val_phone_digits=?11, val_email_required=?12, val_phone_required=?13,
         val_desc_min=?14, val_price_min=?15, val_area_min=?16, default_theme=?17, default_zoom=?18 WHERE id=1",
        params![
            s.show_name as i64, s.show_phone as i64, s.show_email as i64, s.show_price as i64,
            s.show_address as i64, s.show_description as i64, s.show_cover_image as i64,
            s.header_text, s.footer_text,
            s.val_title_min, s.val_phone_digits, s.val_email_required as i64, s.val_phone_required as i64,
            s.val_desc_min, s.val_price_min, s.val_area_min, s.default_theme, s.default_zoom
        ],
    )
    .map_err(|e| internal(&e.to_string()))?;
    let s = conn
        .query_row("SELECT * FROM settings WHERE id = 1", [], map_settings)
        .map_err(|e| internal(&e.to_string()))?;
    Ok(Json(s))
}

// =====================================================================
// DOSYALAR
// =====================================================================

pub async fn serve_file(State(state): State<AppState>, Path(name): Path<String>) -> Response {
    // yol geçişini engelle
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return not_found("Dosya bulunamadı");
    }
    let path = state.upload_dir.join(&name);
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime.as_ref().to_string())],
                bytes,
            )
                .into_response()
        }
        Err(_) => not_found("Dosya bulunamadı"),
    }
}

// =====================================================================
// YEDEKLEME
// =====================================================================

/// Veritabanı ve resimleri içeren bir ZIP yedeği döndürür (yalnızca admin).
pub async fn backup(State(state): State<AppState>, _admin: AdminUser) -> Response {
    let data_dir = state.data_dir.clone();
    let mut cursor = std::io::Cursor::new(Vec::new());
    match build_backup_zip(&data_dir, &mut cursor) {
        Ok(count) => {
            let bytes = cursor.into_inner();
            let filename = format!(
                "gayrimenkul360-yedek-{}.zip",
                chrono::Local::now().format("%Y%m%d-%H%M%S")
            );
            tracing::info!("Yedek oluşturuldu: {count} dosya, {} bayt", bytes.len());
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/zip".to_string()),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{filename}\""),
                    ),
                ],
                bytes,
            )
                .into_response()
        }
        Err(e) => internal(&format!("Yedek oluşturulamadı: {e}")),
    }
}

fn build_backup_zip<W: Write + std::io::Seek>(data_dir: &std::path::Path, writer: W) -> Result<usize, String> {
    let mut zip = zip::ZipWriter::new(writer);
    let opts: zip::write::SimpleFileOptions =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut count = 0usize;

    let db = data_dir.join("gayrimenkul.db");
    if db.is_file() {
        let bytes = std::fs::read(&db).map_err(|e| e.to_string())?;
        zip.start_file("gayrimenkul.db", opts).map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
        count += 1;
    }

    let uploads = data_dir.join("uploads");
    if uploads.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&uploads) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if let Ok(bytes) = std::fs::read(&path) {
                    zip.start_file(format!("uploads/{name}"), opts).map_err(|e| e.to_string())?;
                    zip.write_all(&bytes).map_err(|e| e.to_string())?;
                    count += 1;
                }
            }
        }
    }

    let manifest = format!(
        "Gayrimenkul360 yedek\nTarih: {}\nDosya sayisi: {}\n",
        crate::db::now(),
        count
    );
    zip.start_file("YEDEK-BILGI.txt", opts).map_err(|e| e.to_string())?;
    zip.write_all(manifest.as_bytes()).map_err(|e| e.to_string())?;

    zip.finish().map_err(|e| e.to_string())?;
    Ok(count)
}

// =====================================================================
// Yardımcılar
// =====================================================================

/// Multipart içindeki ilk dosyayı upload klasörüne kaydeder ve dosya adını döndürür.
async fn save_first_upload(state: &AppState, multipart: &mut Multipart) -> ApiResult<String> {
    while let Some(field) = multipart.next_field().await.map_err(|e| bad_request(&e.to_string()))? {
        let orig = field.file_name().unwrap_or("dosya.jpg").to_string();
        let ext = std::path::Path::new(&orig)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg")
            .to_lowercase();
        let data = field.bytes().await.map_err(|e| bad_request(&e.to_string()))?;
        if data.is_empty() {
            continue;
        }
        let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
        let path = state.upload_dir.join(&filename);
        let mut f = std::fs::File::create(&path).map_err(|e| internal(&e.to_string()))?;
        f.write_all(&data).map_err(|e| internal(&e.to_string()))?;
        return Ok(filename);
    }
    Err(bad_request("Dosya bulunamadı"))
}
