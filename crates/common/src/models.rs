use serde::{Deserialize, Serialize};

pub type Id = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Admin,
    Agent,
}

impl Role {
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Agent => "agent",
        }
    }
    pub fn from_str(s: &str) -> Role {
        match s {
            "admin" => Role::Admin,
            _ => Role::Agent,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            Role::Admin => "Yönetici",
            Role::Agent => "Danışman",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Id,
    pub username: String,
    pub full_name: String,
    pub role: Role,
    pub phone: String,
    pub email: String,
    pub photo: Option<String>,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub role: Role,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

// ---------------- Portfoy ----------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ListingType {
    Satilik,
    Kiralik,
}

impl ListingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ListingType::Satilik => "satilik",
            ListingType::Kiralik => "kiralik",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "kiralik" => ListingType::Kiralik,
            _ => ListingType::Satilik,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            ListingType::Satilik => "Satılık",
            ListingType::Kiralik => "Kiralık",
        }
    }
    pub const ALL: [ListingType; 2] = [ListingType::Satilik, ListingType::Kiralik];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyCategory {
    Konut,
    Isyeri,
    Arsa,
    Tarla,
    Turizm,
}

impl PropertyCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyCategory::Konut => "konut",
            PropertyCategory::Isyeri => "isyeri",
            PropertyCategory::Arsa => "arsa",
            PropertyCategory::Tarla => "tarla",
            PropertyCategory::Turizm => "turizm",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "isyeri" => PropertyCategory::Isyeri,
            "arsa" => PropertyCategory::Arsa,
            "tarla" => PropertyCategory::Tarla,
            "turizm" => PropertyCategory::Turizm,
            _ => PropertyCategory::Konut,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            PropertyCategory::Konut => "Konut",
            PropertyCategory::Isyeri => "İş Yeri",
            PropertyCategory::Arsa => "Arsa",
            PropertyCategory::Tarla => "Tarla",
            PropertyCategory::Turizm => "Turizm",
        }
    }
    pub const ALL: [PropertyCategory; 5] = [
        PropertyCategory::Konut,
        PropertyCategory::Isyeri,
        PropertyCategory::Arsa,
        PropertyCategory::Tarla,
        PropertyCategory::Turizm,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortfolioStatus {
    Taslak,
    Yayinda,
    Kiralandi,
    Satildi,
}

impl PortfolioStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PortfolioStatus::Taslak => "taslak",
            PortfolioStatus::Yayinda => "yayinda",
            PortfolioStatus::Kiralandi => "kiralandi",
            PortfolioStatus::Satildi => "satildi",
        }
    }
    pub fn from_str(s: &str) -> Self {
        match s {
            "yayinda" => PortfolioStatus::Yayinda,
            "kiralandi" => PortfolioStatus::Kiralandi,
            "satildi" => PortfolioStatus::Satildi,
            _ => PortfolioStatus::Taslak,
        }
    }
    pub fn label(&self) -> &'static str {
        match self {
            PortfolioStatus::Taslak => "Taslak",
            PortfolioStatus::Yayinda => "Yayında",
            PortfolioStatus::Kiralandi => "Kiralandı",
            PortfolioStatus::Satildi => "Satıldı",
        }
    }
    pub const ALL: [PortfolioStatus; 4] = [
        PortfolioStatus::Taslak,
        PortfolioStatus::Yayinda,
        PortfolioStatus::Kiralandi,
        PortfolioStatus::Satildi,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub id: Id,
    pub title: String,
    pub listing_type: String,
    pub category: String,
    pub status: String,
    pub price: f64,
    pub currency: String,
    pub area_m2: f64,
    #[serde(default)]
    pub area_net: f64,
    #[serde(default)]
    pub ada_parsel: String,
    pub room_count: String,
    pub building_age: String,
    pub floor: String,
    pub total_floors: String,
    pub heating: String,
    pub furnished: bool,
    pub il: String,
    pub ilce: String,
    pub mahalle: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    pub owner_crm_id: Option<Id>,
    pub created_by: Id,
    pub created_by_name: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub images: Vec<String>,
    #[serde(default)]
    pub is_deleted: bool,
    #[serde(default)]
    pub deleted_by_user_id: Option<Id>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioInput {
    pub title: String,
    pub listing_type: String,
    pub category: String,
    pub status: String,
    pub price: f64,
    pub currency: String,
    pub area_m2: f64,
    #[serde(default)]
    pub area_net: f64,
    #[serde(default)]
    pub ada_parsel: String,
    pub room_count: String,
    pub building_age: String,
    pub floor: String,
    pub total_floors: String,
    pub heating: String,
    pub furnished: bool,
    pub il: String,
    pub ilce: String,
    pub mahalle: String,
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    pub owner_crm_id: Option<Id>,
}

// ---------------- CRM ----------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CrmRecord {
    pub id: Id,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub owner_type: String,
    pub notes: String,
    pub created_by: Id,
    pub created_by_name: String,
    pub created_at: String,
    #[serde(default)]
    pub photo: Option<String>,
    #[serde(default)]
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrmInput {
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub owner_type: String,
    pub notes: String,
}

// ---------------- Gorev ----------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Task {
    pub id: Id,
    pub title: String,
    pub description: String,
    pub assigned_to: Option<Id>,
    pub assigned_to_name: String,
    #[serde(default)]
    pub assigned_to_ids: Vec<Id>,
    pub assigned_by: Id,
    pub assigned_by_name: String,
    pub due_at: String,
    pub priority: String,
    pub done: bool,
    pub created_at: String,
    #[serde(default)]
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub title: String,
    pub description: String,
    pub assigned_to: Vec<Id>,
    pub due_at: String,
    pub priority: String,
}

// ---------------- Hedef ----------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Goal {
    pub id: Id,
    pub scope: String,
    pub user_id: Option<Id>,
    pub user_name: String,
    pub title: String,
    pub metric: String,
    pub target_count: i64,
    pub current_count: i64,
    pub period_start: String,
    pub period_end: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalInput {
    pub scope: String,
    pub user_id: Option<Id>,
    pub title: String,
    pub metric: String,
    pub target_count: i64,
    pub period_start: String,
    pub period_end: String,
}

// ---------------- Vardiya Planı ve Mesai Onay ----------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShiftPlan {
    pub id: Id,
    pub user_id: Id,
    pub user_name: String,
    pub month: String, // YYYY-MM
    pub work_days: i64,
    pub daily_hours: f64,
    pub start_time: String,
    pub end_time: String,
    pub note: String,
    /// Haftanın günleri (1=Pazartesi ... 7=Pazar), virgülle ayrılmış. Örn: "1,3,5"
    #[serde(default)]
    pub weekdays: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftPlanInput {
    pub user_id: Id,
    pub month: String,
    pub work_days: i64,
    pub daily_hours: f64,
    pub start_time: String,
    pub end_time: String,
    pub note: String,
    #[serde(default)]
    pub weekdays: String,
}

/// Toplu mesai onayı (yönetici).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkApproveInput {
    pub ids: Vec<Id>,
    pub approved: bool,
}

/// Ekran yardım içeriği (yönetici düzenleyebilir).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpTopic {
    pub key: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShiftAttendance {
    pub id: Id,
    pub user_id: Id,
    pub user_name: String,
    pub date: String, // YYYY-MM-DD
    pub hours: f64,
    pub approved: bool,
    pub approved_by: Option<Id>,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftAttendanceInput {
    pub user_id: Id,
    pub date: String,
    pub hours: f64,
    pub note: String,
}

// ---------------- Bildirim ----------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Notification {
    pub id: Id,
    pub user_id: Id,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub read: bool,
    pub created_at: String,
}

// ---------------- Cikti Ayarlari ----------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub id: Id,
    pub show_name: bool,
    pub show_phone: bool,
    pub show_email: bool,
    pub show_price: bool,
    pub show_address: bool,
    pub show_description: bool,
    pub show_cover_image: bool,
    pub header_text: String,
    pub footer_text: String,
    // --- Form doğrulama kuralları (admin tarafından yönetilir) ---
    #[serde(default = "d_three")]
    pub val_title_min: i64,
    #[serde(default = "d_ten")]
    pub val_phone_digits: i64,
    #[serde(default)]
    pub val_email_required: bool,
    #[serde(default)]
    pub val_phone_required: bool,
    #[serde(default)]
    pub val_desc_min: i64,
    #[serde(default)]
    pub val_price_min: f64,
    #[serde(default)]
    pub val_area_min: f64,
    // --- Varsayılan görünüm (yeni istemciler için) ---
    #[serde(default = "d_system")]
    pub default_theme: String,
    #[serde(default = "d_one_f")]
    pub default_zoom: f64,
}

fn d_three() -> i64 { 3 }
fn d_ten() -> i64 { 10 }
fn d_system() -> String { "system".to_string() }
fn d_one_f() -> f64 { 1.0 }

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            id: 1,
            show_name: true,
            show_phone: true,
            show_email: true,
            show_price: true,
            show_address: true,
            show_description: true,
            show_cover_image: true,
            header_text: "GAYRİMENKUL360".to_string(),
            footer_text: "Bu tanıtım belgesi Gayrimenkul360 tarafından oluşturulmuştur.".to_string(),
            val_title_min: 3,
            val_phone_digits: 10,
            val_email_required: false,
            val_phone_required: false,
            val_desc_min: 0,
            val_price_min: 0.0,
            val_area_min: 0.0,
            default_theme: "system".to_string(),
            default_zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}
