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
    pub assigned_by: Id,
    pub assigned_by_name: String,
    pub due_at: String,
    pub priority: String,
    pub done: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    pub title: String,
    pub description: String,
    pub assigned_to: Option<Id>,
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

// ---------------- Mesai ----------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Shift {
    pub id: Id,
    pub user_id: Id,
    pub user_name: String,
    pub clock_in: String,
    pub clock_out: Option<String>,
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
}

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
}
