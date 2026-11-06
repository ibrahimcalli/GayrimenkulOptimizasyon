use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use common::{CrmRecord, OutputSettings, Portfolio, Role, ShiftAttendance, ShiftPlan, Task, User};
use rusqlite::{Connection, Row};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub upload_dir: Arc<PathBuf>,
    pub data_dir: Arc<PathBuf>,
}

impl AppState {
    pub fn new(conn: Connection, upload_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            db: Arc::new(Mutex::new(conn)),
            upload_dir: Arc::new(upload_dir),
            data_dir: Arc::new(data_dir),
        }
    }
}

// ---------- Row mappers ----------

pub fn map_user(row: &Row) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get("id")?,
        username: row.get("username")?,
        full_name: row.get("full_name")?,
        role: Role::from_str(&row.get::<_, String>("role")?),
        phone: row.get("phone")?,
        email: row.get("email")?,
        photo: row.get("photo")?,
        active: row.get::<_, i64>("active")? != 0,
        created_at: row.get("created_at")?,
    })
}

pub fn map_crm(row: &Row) -> rusqlite::Result<CrmRecord> {
    Ok(CrmRecord {
        id: row.get("id")?,
        full_name: row.get("full_name")?,
        phone: row.get("phone")?,
        email: row.get("email")?,
        address: row.get("address")?,
        owner_type: row.get("owner_type")?,
        notes: row.get("notes")?,
        created_by: row.get("created_by")?,
        created_by_name: row.get::<_, Option<String>>("created_by_name")?.unwrap_or_default(),
        created_at: row.get("created_at")?,
        photo: row.get("photo")?,
        is_deleted: row.get::<_, i64>("is_deleted").unwrap_or(0) != 0,
    })
}

pub fn map_portfolio(row: &Row) -> rusqlite::Result<Portfolio> {
    Ok(Portfolio {
        id: row.get("id")?,
        title: row.get("title")?,
        listing_type: row.get("listing_type")?,
        category: row.get("category")?,
        status: row.get("status")?,
        price: row.get("price")?,
        currency: row.get("currency")?,
        area_m2: row.get("area_m2")?,
        area_net: row.get::<_, Option<f64>>("area_net").unwrap_or(None).unwrap_or(0.0),
        ada_parsel: row.get::<_, Option<String>>("ada_parsel").unwrap_or(None).unwrap_or_default(),
        room_count: row.get("room_count")?,
        building_age: row.get("building_age")?,
        floor: row.get("floor")?,
        total_floors: row.get("total_floors")?,
        heating: row.get("heating")?,
        furnished: row.get::<_, i64>("furnished")? != 0,
        il: row.get("il")?,
        ilce: row.get("ilce")?,
        mahalle: row.get("mahalle")?,
        address: row.get("address")?,
        latitude: row.get("latitude")?,
        longitude: row.get("longitude")?,
        description: row.get("description")?,
        owner_crm_id: row.get("owner_crm_id")?,
        created_by: row.get("created_by")?,
        created_by_name: row.get::<_, Option<String>>("created_by_name")?.unwrap_or_default(),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        images: Vec::new(),
        is_deleted: row.get::<_, i64>("is_deleted").unwrap_or(0) != 0,
        deleted_by_user_id: row.get("deleted_by_user_id").unwrap_or(None),
    })
}

pub fn map_task(row: &Row) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get("id")?,
        title: row.get("title")?,
        description: row.get("description")?,
        assigned_to: row.get("assigned_to")?,
        assigned_to_name: row.get::<_, Option<String>>("assigned_to_name")?.unwrap_or_default(),
        assigned_to_ids: Vec::new(),
        assigned_by: row.get("assigned_by")?,
        assigned_by_name: row.get::<_, Option<String>>("assigned_by_name")?.unwrap_or_default(),
        due_at: row.get("due_at")?,
        priority: row.get("priority")?,
        done: row.get::<_, i64>("done")? != 0,
        created_at: row.get("created_at")?,
        is_deleted: row.get::<_, i64>("is_deleted").unwrap_or(0) != 0,
    })
}

pub fn map_shift_plan(row: &Row) -> rusqlite::Result<ShiftPlan> {
    Ok(ShiftPlan {
        id: row.get("id")?,
        user_id: row.get("user_id")?,
        user_name: row.get::<_, Option<String>>("user_name")?.unwrap_or_default(),
        month: row.get("month")?,
        work_days: row.get("work_days")?,
        daily_hours: row.get("daily_hours")?,
        start_time: row.get("start_time")?,
        end_time: row.get("end_time")?,
        note: row.get("note")?,
        weekdays: row.get::<_, Option<String>>("weekdays").unwrap_or(None).unwrap_or_default(),
    })
}

pub fn map_attendance(row: &Row) -> rusqlite::Result<ShiftAttendance> {
    Ok(ShiftAttendance {
        id: row.get("id")?,
        user_id: row.get("user_id")?,
        user_name: row.get::<_, Option<String>>("user_name")?.unwrap_or_default(),
        date: row.get("date")?,
        hours: row.get("hours")?,
        approved: row.get::<_, i64>("approved")? != 0,
        approved_by: row.get("approved_by")?,
        note: row.get("note")?,
    })
}

pub fn map_settings(row: &Row) -> rusqlite::Result<OutputSettings> {
    Ok(OutputSettings {
        id: row.get("id")?,
        show_name: row.get::<_, i64>("show_name")? != 0,
        show_phone: row.get::<_, i64>("show_phone")? != 0,
        show_email: row.get::<_, i64>("show_email")? != 0,
        show_price: row.get::<_, i64>("show_price")? != 0,
        show_address: row.get::<_, i64>("show_address")? != 0,
        show_description: row.get::<_, i64>("show_description")? != 0,
        show_cover_image: row.get::<_, i64>("show_cover_image")? != 0,
        header_text: row.get("header_text")?,
        footer_text: row.get("footer_text")?,
        val_title_min: row.get("val_title_min")?,
        val_phone_digits: row.get("val_phone_digits")?,
        val_email_required: row.get::<_, i64>("val_email_required")? != 0,
        val_phone_required: row.get::<_, i64>("val_phone_required")? != 0,
        val_desc_min: row.get("val_desc_min")?,
        val_price_min: row.get("val_price_min")?,
        val_area_min: row.get("val_area_min")?,
        default_theme: row.get("default_theme")?,
        default_zoom: row.get("default_zoom")?,
    })
}

/// Portföy kaydına ait resim dosya adlarını yükler.
pub fn load_images(conn: &Connection, portfolio_id: i64) -> Vec<String> {
    let mut stmt = match conn.prepare(
        "SELECT filename FROM portfolio_images WHERE portfolio_id = ?1 ORDER BY sort_order, id",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = stmt.query_map([portfolio_id], |r| r.get::<_, String>(0));
    match rows {
        Ok(it) => it.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}
