use std::collections::HashMap;
use std::path::PathBuf;

use chrono::Datelike;
use common::discovery::DiscoveredServer;
use common::*;
use eframe::egui;

use crate::api::Api;
use crate::export;
use crate::export::PortfolioLabels;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Login,
    Welcome,
    PortfolioForm,
    PortfolioDetail,
    Tasks,
    Goals,
    Shifts,
    Crm,
    Users,
    Settings,
    Profile,
}

#[derive(Clone, Default)]
pub struct FormState {
    pub edit_id: Option<i64>,
    pub title: String,
    pub listing_type: usize,
    pub category: usize,
    pub status: usize,
    pub price: f64,
    pub currency: usize,
    pub price_text: String,
    pub area_m2: f64,
    pub area_net: f64,
    pub ada_parsel: String,
    pub coord_paste: String,
    pub room_count: String,
    pub building_age: String,
    pub floor: String,
    pub total_floors: String,
    pub heating_choices: Vec<usize>,
    pub owner_type_choice: usize,
    pub furnished: bool,
    pub ilce: usize,    // 0 = seçilmedi, diğer: DISTRICTS[i-1]
    pub mahalle: usize, // 0 = seçilmedi
    pub address: String,
    pub other_il: bool,
    pub il_custom: String,
    pub latitude: f64,
    pub longitude: f64,
    pub description: String,
    pub owner_choice: usize, // 0 yok, 1 yeni, 2+ mevcut CRM
    pub new_owner_name: String,
    pub new_owner_phone: String,
    pub new_owner_email: String,
    pub new_owner_type: String,
    pub new_owner_notes: String,
    pub new_images: Vec<PathBuf>,
    pub existing_images: Vec<String>,
}

#[derive(Clone, Default)]
pub struct UserForm {
    pub edit_id: Option<i64>,
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub role_is_admin: bool,
    pub phone: String,
    pub email: String,
}

#[derive(Clone, Default)]
pub struct TaskForm {
    pub title: String,
    pub description: String,
    pub assignee: usize, // 0 = tüm ekip, diğer: users[i-1]
    pub due_date: String,
    pub due_time: String,
    pub priority: usize,
}

#[derive(Clone, Default)]
pub struct GoalForm {
    pub title: String,
    pub metric: usize,
    pub target_count: i64,
    pub scope_user: usize, // 0 = tüm ekip, diğer: users[i-1]
    pub period_start: String,
    pub period_end: String,
}

#[derive(Clone, Default)]
pub struct CrmForm {
    pub edit_id: Option<i64>,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub address: String,
    pub owner_type: String,
    pub notes: String,
}

#[derive(Clone, Default)]
pub struct ShiftPlanForm {
    pub user_idx: usize,
    pub month: String,
    pub weekdays: Vec<usize>, // 1=Pzt .. 7=Paz
    pub start_time: String,
    pub end_time: String,
    pub note: String,
}

#[derive(Clone, Default)]
pub struct ShiftAttendanceForm {
    pub user_idx: usize,
    pub date: String,
    pub hours: f64,
    pub note: String,
}

pub struct App {
    pub api: Api,
    pub screen: Screen,
    pub user: Option<User>,
    pub users: Vec<User>,
    pub portfolios: Vec<Portfolio>,
    pub tasks: Vec<Task>,
    pub goals: Vec<Goal>,
    pub plans: Vec<ShiftPlan>,
    pub attendance: Vec<ShiftAttendance>,
    pub shift_tab: usize,
    pub card_scale: f32,
    pub show_deleted: bool,
    pub crm_list_view: bool,
    pub task_assignees_sel: Vec<i64>,
    pub crm: Vec<CrmRecord>,
    pub notifications: Vec<Notification>,
    pub settings: OutputSettings,

    pub textures: HashMap<String, egui::TextureHandle>,

    pub server_url: String,
    pub login_username: String,
    pub login_password: String,
    pub login_error: Option<String>,

    pub error: Option<String>,
    pub info: Option<String>,
    pub toast: Option<(String, bool, f64)>,

    pub form: FormState,
    pub user_form: UserForm,
    pub task_form: TaskForm,
    pub goal_form: GoalForm,
    pub crm_form: CrmForm,
    pub plan_form: ShiftPlanForm,
    pub att_form: ShiftAttendanceForm,

    pub profile_name: String,
    pub profile_phone: String,
    pub profile_email: String,
    pub old_password: String,
    pub new_password: String,

    pub search: String,
    pub filter_ilce: usize,
    pub filter_listing: usize,
    pub filter_status: usize,
    pub list_view: bool,
    pub only_mine: bool,
    pub detail_id: Option<i64>,
    pub cover_zoom: Option<(String, f64)>,

    pub show_notifications: bool,
    pub last_notification_poll: f64,

    pub discovered: Vec<DiscoveredServer>,
    pub discovering: bool,
    pub connected: bool,
    pub last_health: f64,
    pub last_auto_refresh: f64,

    // Görünüm (yerel tercihler)
    pub theme: String, // "dark" | "light" | "system"
    pub zoom: f64,
    pub local_theme_set: bool,
    pub settings_tab: SettingsTab,
    pub font_family: usize, // 0=Roboto 1=Sistem
    pub font_size: f64,     // gövde punto
    pub accent: String,     // "blue"|"green"|"gray"|"orange"|"purple"
    pub att_sel: Vec<i64>,
    pub report_sel: Vec<i64>,
    pub report_month: String,
    pub help_topics: Vec<HelpTopic>,
    pub help_open: Option<String>,
    pub test_log: String,
    pub test_running: bool,
}

/// Yeni boş portföy formu; "Yeni sahip ekle" varsayılan seçili gelir.
pub fn fresh_form() -> FormState {
    FormState {
        owner_choice: 1,
        status: 0, // PortfolioStatus::ALL[0] = Aktif
        ..Default::default()
    }
}

/// Fiyat girişinde binlik ayraç otomatik: 6000000 -> 6.000.000
pub fn format_price_input(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).take(15).collect();
    if digits.is_empty() {
        return String::new();
    }
    let mut out = String::new();
    for (i, ch) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// Yapıştırılan metinden "36.1234, 29.5678" biçiminde koordinat çıkarır.
pub fn parse_coords(s: &str) -> Option<(f64, f64)> {
    let cleaned: String = s
        .chars()
        .map(|c| match c { '٫' | ',' => ',', _ => c })
        .collect();
    let nums: Vec<&str> = cleaned
        .split(|c: char| !(c.is_ascii_digit() || c == '.' || c == ',' || c == '-'))
        .filter(|x| !x.is_empty())
        .collect();
    if nums.len() < 2 {
        return None;
    }
    let lat: f64 = nums[0].replace(',', ".").parse().ok()?;
    let lng: f64 = nums[1].replace(',', ".").parse().ok()?;
    if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
        return None;
    }
    Some((lat, lng))
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    View,
    Output,
    Validation,
    Users,
    Help,
    Test,
}

const CURRENCIES: [&str; 4] = ["TRY", "USD", "EUR", "GBP"];
const HEATINGS: [&str; 6] = ["Kombi", "Merkezi", "Soba", "Klima", "Yerden Isıtma", "Yok"];
const OWNER_TYPES: [&str; 3] = ["Sahip", "Yetkili", "Müşteri"];
const PRIORITIES: [&str; 3] = ["düşük", "normal", "yüksek"];
const METRICS: [&str; 5] = [
    "Portföy tamamlama",
    "Yayına alma",
    "Kiralama yapma",
    "Satış yapma",
    "Toplam portföy",
];

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Türkçe karakter desteği: Roboto fontlarını varsayılan yap.
        Self::install_roboto_fonts(&cc.egui_ctx);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        cc.egui_ctx.set_style(style);

        let cfg = load_config();
        let server_url = if cfg.server_url.is_empty() {
            "http://127.0.0.1:8080".to_string()
        } else {
            cfg.server_url
        };

        let mut app = Self {
            api: Api::new(server_url.clone()),
            screen: Screen::Login,
            user: None,
            users: Vec::new(),
            portfolios: Vec::new(),
            tasks: Vec::new(),
            goals: Vec::new(),
            plans: Vec::new(),
            attendance: Vec::new(),
            shift_tab: 0,
            card_scale: 1.0,
            show_deleted: false,
            crm_list_view: false,
            task_assignees_sel: Vec::new(),
            crm: Vec::new(),
            notifications: Vec::new(),
            settings: OutputSettings::default(),
            textures: HashMap::new(),
            server_url,
            login_username: String::new(),
            login_password: String::new(),
            login_error: None,
            error: None,
            info: None,
            toast: None,
            form: FormState::default(),
            user_form: UserForm::default(),
            task_form: TaskForm::default(),
            goal_form: GoalForm::default(),
            crm_form: CrmForm::default(),
            plan_form: ShiftPlanForm::default(),
            att_form: ShiftAttendanceForm::default(),
            profile_name: String::new(),
            profile_phone: String::new(),
            profile_email: String::new(),
            old_password: String::new(),
            new_password: String::new(),
            search: String::new(),
            filter_ilce: 0,
            filter_listing: 0,
            filter_status: 0,
            list_view: false,
            only_mine: false,
            detail_id: None,
            cover_zoom: None,
            show_notifications: false,
            last_notification_poll: 0.0,
            discovered: Vec::new(),
            discovering: false,
            connected: false,
            last_health: 0.0,
            last_auto_refresh: 0.0,
            theme: if cfg.theme.is_empty() { "system".to_string() } else { cfg.theme.clone() },
            zoom: if cfg.zoom > 0.05 { cfg.zoom } else { 1.0 },
            local_theme_set: !cfg.theme.is_empty(),
            settings_tab: SettingsTab::View,
            font_family: cfg.font_family,
            font_size: if (10.0..=26.0).contains(&cfg.font_size) { cfg.font_size } else { 14.0 },
            accent: if cfg.accent.is_empty() { "blue".to_string() } else { cfg.accent },
            att_sel: Vec::new(),
            report_sel: Vec::new(),
            report_month: String::new(),
            help_topics: Vec::new(),
            help_open: None,
            test_log: String::new(),
            test_running: false,
        };
        app.apply_all_ui(&cc.egui_ctx);
        app
    }

    /// Tema + font + ölçek + renk vurgusu tercihlerinin tamamını uygular.
    fn apply_all_ui(&self, ctx: &egui::Context) {
        self.apply_theme(ctx);
        self.apply_polish(ctx);
        self.apply_fonts(ctx);
        self.apply_font_size(ctx);
        self.apply_accent(ctx);
        ctx.set_zoom_factor(self.zoom as f32);
    }

    /// Modern sistem standardı görünüm: yumuşak köşeler, nefes alan boşluklar, katmanlı yüzeyler.
    fn apply_polish(&self, ctx: &egui::Context) {
        let mut st = (*ctx.style()).clone();
        st.spacing.button_padding = egui::vec2(10.0, 6.0);
        st.spacing.item_spacing = egui::vec2(10.0, 8.0);
        st.spacing.menu_margin = egui::Margin::symmetric(10.0, 8.0);
        st.spacing.indent = 18.0;
        let r = egui::Rounding::same(6.0);
        st.visuals.window_rounding = r;
        st.visuals.menu_rounding = r;
        st.visuals.widgets.noninteractive.rounding = r;
        st.visuals.widgets.inactive.rounding = r;
        st.visuals.widgets.hovered.rounding = r;
        st.visuals.widgets.active.rounding = r;
        st.visuals.widgets.open.rounding = r;
        let dark = st.visuals.dark_mode;
        st.visuals.extreme_bg_color = if dark { egui::Color32::from_gray(24) } else { egui::Color32::from_gray(243) }; // TextEdit arka planı extreme_bg'den gelir
        st.visuals.faint_bg_color = if dark { egui::Color32::from_gray(32) } else { egui::Color32::from_gray(248) };
        st.visuals.widgets.inactive.weak_bg_fill = if dark { egui::Color32::from_gray(45) } else { egui::Color32::from_gray(235) };
        ctx.set_style(st);
    }

    /// Yazı tipi ailesini uygular (Roboto gömülü / sistem fontu).
    fn apply_fonts(&self, ctx: &egui::Context) {
        match self.font_family {
            1 => {
                let mut f = egui::FontDefinitions::default();
                f.families.insert(
                    egui::FontFamily::Name("navbold".into()),
                    vec!["Ubuntu-Light".to_string()],
                );
                ctx.set_fonts(f);
            }
            _ => Self::install_roboto_fonts(ctx),
        }
    }

    /// Gövde punto boyutunu tüm metin stillerine oranlı uygular.
    fn apply_font_size(&self, ctx: &egui::Context) {
        let base = self.font_size as f32;
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::proportional(base * 1.45)),
            (egui::TextStyle::Body, egui::FontId::proportional(base)),
            (egui::TextStyle::Button, egui::FontId::proportional(base)),
            (egui::TextStyle::Small, egui::FontId::proportional(base * 0.8)),
            (egui::TextStyle::Monospace, egui::FontId::monospace(base * 0.9)),
        ]
        .into();
        ctx.set_style(style);
    }

    /// Renk vurgusu (mavi/yeşil/gri/turuncu/mor) uygular.
    fn apply_accent(&self, ctx: &egui::Context) {
        let accent = match self.accent.as_str() {
            "green" => egui::Color32::from_rgb(56, 142, 60),
            "gray" => egui::Color32::from_rgb(120, 128, 136),
            "orange" => egui::Color32::from_rgb(224, 122, 40),
            "purple" => egui::Color32::from_rgb(140, 68, 173),
            _ => egui::Color32::from_rgb(46, 116, 181),
        };
        let mut style = (*ctx.style()).clone();
        style.visuals.selection.bg_fill = accent;
        style.visuals.selection.stroke = egui::Stroke::new(1.2, egui::Color32::WHITE);
        style.visuals.widgets.active.bg_fill = accent;
        style.visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.35);
        style.visuals.hyperlink_color = accent;
        ctx.set_style(style);
    }

    /// Tema tercihini egui'ye uygular.
    fn apply_theme(&self, ctx: &egui::Context) {
        let pref = match self.theme.as_str() {
            "dark" => egui::ThemePreference::Dark,
            "light" => egui::ThemePreference::Light,
            _ => egui::ThemePreference::System,
        };
        ctx.set_theme(pref);
    }

    /// Yakınlaştırma (UI ölçek) değerini uygular ve yerel yapılandırmaya kaydeder.
    fn set_zoom(&mut self, ctx: &egui::Context, zoom: f64) {
        let z = zoom.clamp(0.6, 2.5);
        self.zoom = z;
        ctx.set_zoom_factor(z as f32);
        self.save_config();
    }

    /// Ctrl +/−/0 kısayolları ile dinamik ölçeklendirme.
    fn handle_zoom_shortcuts(&mut self, ctx: &egui::Context) {
        if !ctx.input(|i| i.modifiers.ctrl) {
            return;
        }
        let (plus, minus, reset) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus),
                i.key_pressed(egui::Key::Minus),
                i.key_pressed(egui::Key::Num0),
            )
        });
        if plus {
            self.set_zoom(ctx, self.zoom + 0.1);
        } else if minus {
            self.set_zoom(ctx, self.zoom - 0.1);
        } else if reset {
            self.set_zoom(ctx, 1.0);
        }
    }

    fn is_admin(&self) -> bool {
        self.user.as_ref().map(|u| u.role == Role::Admin).unwrap_or(false)
    }

    fn toast(&mut self, msg: impl Into<String>, is_error: bool) {
        self.toast = Some((msg.into(), is_error, now_secs()));
    }

    fn set_error(&mut self, e: String) {
        self.error = Some(e);
    }

    fn clear_messages(&mut self) {
        self.error = None;
        self.info = None;
    }

    fn save_config(&self) {
        let cfg = ClientConfig {
            server_url: self.server_url.clone(),
            theme: self.theme.clone(),
            zoom: self.zoom,
            font_family: self.font_family,
            font_size: self.font_size,
            accent: self.accent.clone(),
        };
        if let Ok(s) = serde_json::to_string_pretty(&cfg) {
            let _ = std::fs::write(config_file(), s);
        }
    }

    /// Yerel kayıtlı tercih yoksa sunucudaki varsayılan görünümü uygular.
    fn apply_server_ui_defaults(&mut self, ctx: &egui::Context) {
        if self.local_theme_set {
            return;
        }
        let s = self.settings.clone();
        if !s.default_theme.is_empty() {
            self.theme = s.default_theme.clone();
        }
        if s.default_zoom > 0.05 {
            self.zoom = s.default_zoom;
        }
        self.local_theme_set = true;
        self.apply_theme(ctx);
        ctx.set_zoom_factor(self.zoom as f32);
        self.save_config();
    }

    // ---------------- veri yükleme ----------------

    fn load_all(&mut self) {
        self.load_portfolios();
        self.load_users();
        self.load_tasks();
        self.load_goals();
        self.load_plans();
        self.load_attendance();
        self.load_notifications();
        self.load_settings();
        self.load_help();
        if self.is_admin() {
            self.load_crm();
        }
    }

    fn load_help(&mut self) {
        if let Ok(h) = self.api.list_help() {
            self.help_topics = h;
        }
    }

    fn load_portfolios(&mut self) {
        let ilce = if self.filter_ilce == 0 { None } else { Some(common::mugla::DISTRICTS[self.filter_ilce - 1]) };
        let listing = match self.filter_listing {
            1 => Some("satilik"),
            2 => Some("kiralik"),
            _ => None,
        };
        let status = match self.filter_status {
            1 => Some("aktif"),
            2 => Some("yayinda"),
            3 => Some("kiralandi"),
            4 => Some("satildi"),
            5 => Some("taslak"),
            _ => None,
        };
        let owner = if self.only_mine { self.user.as_ref().map(|u| u.id) } else { None };
        let q = if self.search.trim().is_empty() { None } else { Some(self.search.trim()) };
        match self.api.list_portfolios(q, ilce, None, listing, None, status, owner, self.show_deleted) {
            Ok(list) => self.portfolios = list,
            Err(e) => self.set_error(e),
        }
    }

    fn load_users(&mut self) {
        if let Ok(u) = self.api.list_users() {
            self.users = u;
        }
    }
    fn load_tasks(&mut self) {
        match self.api.list_tasks() {
            Ok(t) => self.tasks = t,
            Err(e) => self.set_error(e),
        }
    }
    fn load_goals(&mut self) {
        match self.api.list_goals() {
            Ok(g) => self.goals = g,
            Err(e) => self.set_error(e),
        }
    }
    fn load_plans(&mut self) {
        match self.api.list_shift_plans() {
            Ok(p) => self.plans = p,
            Err(e) => self.set_error(e),
        }
    }
    fn load_attendance(&mut self) {
        match self.api.list_attendance() {
            Ok(a) => self.attendance = a,
            Err(e) => self.set_error(e),
        }
    }
    fn load_notifications(&mut self) {
        if let Ok(n) = self.api.list_notifications() {
            self.notifications = n;
        }
    }
    fn load_crm(&mut self) {
        match self.api.list_crm(self.show_deleted) {
            Ok(c) => self.crm = c,
            Err(e) => self.set_error(e),
        }
    }
    fn load_settings(&mut self) {
        if let Ok(s) = self.api.get_settings() {
            self.settings = s;
        }
    }

    fn navigate(&mut self, s: Screen) {
        self.clear_messages();
        self.screen = s;
        match s {
            Screen::Welcome => self.load_portfolios(),
            Screen::Tasks => self.load_tasks(),
            Screen::Goals => self.load_goals(),
            Screen::Shifts => { self.load_plans(); self.load_attendance(); }
            Screen::Crm => self.load_crm(),
            Screen::Users => self.load_users(),
            Screen::Settings => self.load_settings(),
            Screen::PortfolioForm => {}
            Screen::Profile => {
                if let Some(u) = &self.user {
                    self.profile_name = u.full_name.clone();
                    self.profile_phone = u.phone.clone();
                    self.profile_email = u.email.clone();
                }
            }
            _ => {}
        }
    }

    fn check_login(&mut self) -> bool {
        if self.user.is_none() {
            self.api.token = None;
            self.screen = Screen::Login;
            false
        } else {
            true
        }
    }

    // ---------------- resim dokusu ----------------

    fn texture(&mut self, ctx: &egui::Context, name: &str) -> Option<egui::TextureHandle> {
        if name.is_empty() {
            return None;
        }
        if let Some(t) = self.textures.get(name) {
            return Some(t.clone());
        }
        let bytes = self.api.download_file(name).ok()?;
        let img = image::load_from_memory(&bytes).ok()?;
        let thumb = img.thumbnail(480, 480).to_rgba8();
        let (w, h) = thumb.dimensions();
        let color = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], thumb.as_raw());
        let tex = ctx.load_texture(name.to_string(), color, egui::TextureOptions::LINEAR);
        self.textures.insert(name.to_string(), tex.clone());
        Some(tex)
    }

    fn user_name(&self, id: i64) -> String {
        self.users
            .iter()
            .find(|u| u.id == id)
            .map(|u| {
                if u.full_name.is_empty() {
                    u.username.clone()
                } else {
                    u.full_name.clone()
                }
            })
            .unwrap_or_else(|| "-".to_string())
    }

    // ---------------- update ----------------

    pub const WEEKDAYS: [&str; 7] = ["Pazartesi", "Salı", "Çarşamba", "Perşembe", "Cuma", "Cumartesi", "Pazar"];

fn digits_only(s: &str, max: usize) -> String {
    s.chars().filter(|c| c.is_ascii_digit()).take(max).collect()
}

/// GG-AA-YYYY girişinde her 2 rakamdan sonra otomatik tire ekler.
pub fn auto_dash_date(s: &str) -> String {
    let d = Self::digits_only(s, 8);
    let mut out = String::new();
    for (i, ch) in d.chars().enumerate() {
        if i == 2 || i == 4 {
            out.push('-');
        }
        out.push(ch);
    }
    out
}

/// GG-AA-YYYY -> YYYY-MM-DD (geçersizse None).
pub fn iso_from_display(s: &str) -> Option<String> {
    let p: Vec<&str> = s.split('-').collect();
    if p.len() != 3 {
        return None;
    }
    let (d, m, y) = (p[0], p[1], p[2]);
    let ok_digits = |x: &str| !x.is_empty() && x.chars().all(|c| c.is_ascii_digit());
    if d.len() != 2 || m.len() != 2 || y.len() != 4 || !ok_digits(d) || !ok_digits(m) || !ok_digits(y) {
        return None;
    }
    let dd: u32 = d.parse().ok()?;
    let mm: u32 = m.parse().ok()?;
    let yy: i32 = y.parse().ok()?;
    if !(1..=31).contains(&dd) || !(1..=12).contains(&mm) {
        return None;
    }
    Some(format!("{yy:04}-{mm:02}-{dd:02}"))
}

fn date_field(ui: &mut egui::Ui, value: &mut String, hint: &str) {
    let before = value.clone();
    ui.add(egui::TextEdit::singleline(value).desired_width(104.0).hint_text(hint));
    if *value != before {
        *value = Self::auto_dash_date(value);
    }
}

/// "1,3,5" -> "Pazartesi, Çarşamba, Cuma"
pub fn weekday_names(csv: &str) -> String {
    let mut names = Vec::new();
    for part in csv.split(',') {
        if let Ok(i) = part.trim().parse::<usize>() {
            if (1..=7).contains(&i) {
                names.push(Self::WEEKDAYS[i - 1]);
            }
        }
    }
    if names.is_empty() { "—".to_string() } else { names.join(", ") }
}

/// "10:30" -> 10.5
fn parse_hhmm(s: &str) -> f64 {
    let p: Vec<&str> = s.split(':').collect();
    let h = p.first().and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(0.0);
    let m = p.get(1).and_then(|x| x.trim().parse::<f64>().ok()).unwrap_or(0.0);
    h + m / 60.0
}

/// "2026-11" ayının tüm günleri.
fn month_days(month: &str) -> Vec<chrono::NaiveDate> {
    let p: Vec<&str> = month.split('-').collect();
    if p.len() != 2 {
        return Vec::new();
    }
    let (y, m) = match (p[0].parse::<i32>(), p[1].parse::<u32>()) {
        (Ok(y), Ok(m)) if (1..=12).contains(&m) => (y, m),
        _ => return Vec::new(),
    };
    let first = match chrono::NaiveDate::from_ymd_opt(y, m, 1) {
        Some(d) => d,
        None => return Vec::new(),
    };
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let last = match chrono::NaiveDate::from_ymd_opt(ny, nm, 1) {
        Some(d) => d.pred_opt().unwrap_or(first),
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    let mut cur = first;
    while cur <= last {
        out.push(cur);
        match cur.succ_opt() {
            Some(n) => cur = n,
            None => break,
        }
    }
    out
}

/// Türkçe karakter kümesini (ç ğ ı ö ş ü İ Ğ Ş) tam destekleyen font kurulumu.
/// Roboto tüm metinlerde öncelikli kullanılır; emoji/ikon fontları yedekte kalır.
fn install_roboto_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "roboto_regular".to_string(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")),
    );
    fonts.font_data.insert(
        "roboto_bold".to_string(),
        egui::FontData::from_static(include_bytes!("../../../assets/fonts/Roboto-Bold.ttf")),
    );

    // Proportional ailesinde Roboto'yu en başa al (başlıklar Bold'la, gövde Regular'la)
    let prop = fonts.families.entry(egui::FontFamily::Proportional).or_default();
    prop.insert(0, "roboto_bold".to_string());
    prop.insert(0, "roboto_regular".to_string());

    let mono = fonts.families.entry(egui::FontFamily::Monospace).or_default();
    mono.push("roboto_regular".to_string());

    // Menü için bold aile
    fonts.families.insert(
        egui::FontFamily::Name("navbold".into()),
        vec!["roboto_bold".to_string(), "roboto_regular".to_string()],
    );

    ctx.set_fonts(fonts);
}


    /// Ağ durumunu ve otomatik senkronizasyonu yönetir.
    fn tick_sync(&mut self, ctx: &egui::Context) {
        let t = ctx.input(|i| i.time);
        if t - self.last_health > 8.0 {
            self.last_health = t;
            self.connected = self.api.health();
        }
        // Karşılama ekranındayken diğer kullanıcıların eklediği portföyler
        // otomatik olarak periyodik yenilenir (senkronizasyon).
        if self.user.is_some() && self.screen == Screen::Welcome && t - self.last_auto_refresh > 15.0 {
            self.last_auto_refresh = t;
            self.load_portfolios();
        }
    }

    fn poll_notifications(&mut self, ctx: &egui::Context) {
        if self.user.is_none() {
            return;
        }
        let t = ctx.input(|i| i.time);
        if t - self.last_notification_poll > 20.0 {
            self.last_notification_poll = t;
            self.load_notifications();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_notifications(ctx);
        self.tick_sync(ctx);
        self.handle_zoom_shortcuts(ctx);
        self.handle_card_zoom(ctx);

        // Sürükle-bırak ile resim ekleme (portföy formunda)
        if self.screen == Screen::PortfolioForm {
            let dropped: Vec<PathBuf> = ctx.input(|i| {
                i.raw
                    .dropped_files
                    .iter()
                    .filter_map(|f| f.path.clone())
                    .collect()
            });
            if !dropped.is_empty() {
                self.form.new_images.extend(dropped);
            }
        }

        if self.user.is_some() {
            self.top_bar(ctx);
            self.side_panel(ctx);
        }

        let panel_frame = egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::symmetric(16.0, 12.0));
        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            match self.screen {
                Screen::Login => self.login_screen(ui),
                _ => {
                    if !self.check_login() {
                        self.login_screen(ui);
                        return;
                    }
                    match self.screen {
                        Screen::Welcome => self.welcome_screen(ui, ctx),
                        Screen::PortfolioForm => self.portfolio_form_screen(ui, ctx),
                        Screen::PortfolioDetail => self.detail_screen(ui, ctx),
                        Screen::Tasks => self.tasks_screen(ui),
                        Screen::Goals => self.goals_screen(ui),
                        Screen::Shifts => self.shifts_screen(ui),
                        Screen::Crm => self.crm_screen(ui, ctx),
                        Screen::Users => self.users_screen(ui),
                        Screen::Settings => self.settings_screen(ui),
                        Screen::Profile => self.profile_screen(ui, ctx),
                        Screen::Login => {}
                    }
                }
            }
        });

        // Sağ üst köşe yardım düğmesi
        if self.user.is_some() && self.screen != Screen::Login {
            let key = self.help_key_for(self.screen);
            egui::Area::new(egui::Id::new("help_btn_area"))
                .anchor(egui::Align2::RIGHT_TOP, [-10.0, 34.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let btn = egui::Button::new(
                        egui::RichText::new("?").strong().size(16.0).color(egui::Color32::WHITE),
                    )
                    .fill(egui::Color32::from_rgb(46, 116, 181))
                    .rounding(egui::Rounding::same(14.0))
                    .min_size(egui::vec2(28.0, 28.0));
                    if ui.add(btn).clicked() {
                        self.help_open = Some(key.to_string());
                    }
                });
        }
        self.help_window(ctx);

        self.messages(ctx);
        self.notification_popup(ctx);
        self.cover_zoom_window(ctx);
    }
}

impl App {
    fn help_key_for(&self, screen: Screen) -> &'static str {
        match screen {
            Screen::Welcome => "welcome",
            Screen::PortfolioForm => "portfolio_form",
            Screen::PortfolioDetail => "portfolio_detail",
            Screen::Tasks => "tasks",
            Screen::Goals => "goals",
            Screen::Shifts => "shifts",
            Screen::Crm => "crm",
            Screen::Users => "users",
            Screen::Settings => "settings",
            Screen::Profile => "profile",
            Screen::Login => "welcome",
        }
    }

    fn help_window(&mut self, ctx: &egui::Context) {
        if let Some(key) = self.help_open.clone() {
            let topic = self.help_topics.iter().find(|t| t.key == key).cloned();
            let mut open = true;
            egui::Window::new(
                egui::RichText::new(format!("ℹ  {}", topic.as_ref().map(|t| t.title.clone()).unwrap_or_else(|| "Yardım".into())))
                    .strong(),
            )
            .open(&mut open)
            .default_width(430.0)
            .show(ctx, |ui| {
                match &topic {
                    Some(t) => {
                        // Etiketler seçilebilir: kopyalanabilir
                        for line in t.body.split('\n') {
                            ui.label(egui::RichText::new(line));
                        }
                    }
                    None => {
                        ui.label("Bu ekran için yardım içeriği bulunamadı. Ayarlar > Yardım İçerikleri bölümünden ekleyebilirsiniz.");
                    }
                }
            });
            if !open {
                self.help_open = None;
            }
        }
    }
}

// =====================================================================
//  LOGIN
// =====================================================================

impl App {
    fn login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.heading(egui::RichText::new("İÇ SOFTWARE").size(34.0).strong());
            ui.label("Ofis Portföy ve Müşteri Yönetim Sistemi  •  Gayrimenkul360 motoru");
            ui.add_space(24.0);

            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_max_width(380.0);
                ui.vertical(|ui| {
                    ui.label("Sunucu Adresi (ofis bilgisayarı):");
                    ui.text_edit_singleline(&mut self.server_url);
                    ui.horizontal(|ui| {
                        if ui.button("🔍 Ağdaki Sunucuları Bul").clicked() {
                            self.discovered =
                                crate::api::discover_servers(std::time::Duration::from_millis(1500));
                            if self.discovered.is_empty() {
                                self.login_error = Some(
                                    "Ağda otomatik sunucu bulunamadı. IP adresini elle girebilirsiniz."
                                        .to_string(),
                                );
                            } else {
                                self.login_error = None;
                            }
                        }
                    });
                    if !self.discovered.is_empty() {
                        ui.label(egui::RichText::new("Bulunan sunucular (seçmek için tıklayın):").small().strong());
                        for s in self.discovered.clone() {
                            if ui.selectable_label(self.server_url == s.url(), s.label()).clicked() {
                                self.server_url = s.url();
                            }
                        }
                    }
                    ui.add_space(8.0);
                    ui.label("Kullanıcı Adı:");
                    ui.text_edit_singleline(&mut self.login_username);
                    ui.label("Parola:");
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.login_password).password(true),
                    );
                    ui.add_space(12.0);
                    let submit = ui
                        .add(egui::Button::new(egui::RichText::new("Giriş Yap").size(16.0)).min_size(egui::vec2(120.0, 34.0)))
                        .clicked()
                        || (resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                    if submit {
                        self.do_login(ui.ctx());
                    }
                    if let Some(e) = &self.login_error {
                        ui.add_space(6.0);
                        ui.colored_label(egui::Color32::from_rgb(220, 70, 70), e);
                    }
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("İlk giriş: admin / admin123  (ilk girişten sonra değiştirin)")
                            .small()
                            .weak(),
                    );
                });
            });
        });
    }

    fn do_login(&mut self, ctx: &egui::Context) {
        self.login_error = None;
        self.api.set_base(&self.server_url);
        match self.api.login(self.login_username.trim(), &self.login_password) {
            Ok(auth) => {
                self.save_config();
                self.user = Some(auth.user.clone());
                self.login_password.clear();
                self.load_all();
                self.toast(format!("Hoş geldiniz, {}", auth.user.full_name), false);
                // Yerel tercih kayıtlı değilse sunucu varsayılanlarını uygula
                self.navigate(Screen::Welcome);
                self.apply_server_ui_defaults(ctx);
                self.apply_server_ui_defaults(ctx);
            }
            Err(e) => self.login_error = Some(e),
        }
    }

    fn logout(&mut self) {
        self.api.logout();
        self.api.token = None;
        self.user = None;
        self.textures.clear();
        self.screen = Screen::Login;
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        let unread = self.notifications.iter().filter(|n| !n.read).count();
        let full_name = self.user.as_ref().map(|u| u.full_name.clone()).unwrap_or_default();
        let role = self.user.as_ref().map(|u| u.role).unwrap_or(Role::Agent);
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("İÇ SOFTWARE").color(egui::Color32::from_rgb(46, 116, 181)).strong());
                ui.separator();
                if self.connected {
                    ui.colored_label(egui::Color32::from_rgb(60, 170, 90), "● Bağlı");
                } else {
                    ui.colored_label(egui::Color32::from_rgb(220, 80, 80), "● Bağlantı yok");
                }
                let bell = format!("Bildirimler ({unread})");
                if ui.button(bell).clicked() {
                    self.show_notifications = !self.show_notifications;
                    if self.show_notifications {
                        let _ = self.api.read_all_notifications();
                        self.load_notifications();
                    }
                }
                if ui.button("Yenile").clicked() {
                    self.load_portfolios();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Çıkış").clicked() {
                        self.logout();
                    }
                    ui.label(format!("{}  ({})", full_name, role.label()));
                });
            });
        });
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        let mut target: Option<Screen> = None;
        egui::SidePanel::left("nav")
            .resizable(false)
            .default_width(190.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                nav_button(ui, &mut target, self.screen, Screen::Welcome, "🏢", "Portföyler", egui::Color32::from_rgb(86, 156, 214));
                nav_button(ui, &mut target, self.screen, Screen::PortfolioForm, "➕", "Yeni Portföy", egui::Color32::from_rgb(106, 190, 110));
                nav_button(ui, &mut target, self.screen, Screen::Tasks, "✅", "Görevler", egui::Color32::from_rgb(230, 160, 66));
                nav_button(ui, &mut target, self.screen, Screen::Goals, "🎯", "Hedefler", egui::Color32::from_rgb(186, 110, 220));
                nav_button(ui, &mut target, self.screen, Screen::Shifts, "🕒", "Vardiya / Mesai", egui::Color32::from_rgb(80, 190, 190));
                ui.separator();
                if self.is_admin() {
                    nav_button(ui, &mut target, self.screen, Screen::Crm, "👥", "CRM Kayıtları", egui::Color32::from_rgb(120, 140, 220));
                    nav_button(ui, &mut target, self.screen, Screen::Users, "🧑", "Kullanıcılar", egui::Color32::from_rgb(200, 130, 180));
                    ui.separator();
                }
                nav_button(ui, &mut target, self.screen, Screen::Settings, "⚙", "Ayarlar", egui::Color32::from_rgb(160, 160, 170));
                nav_button(ui, &mut target, self.screen, Screen::Profile, "👤", "Profilim", egui::Color32::from_rgb(150, 190, 120));
                ui.add_space(10.0);
                ui.separator();
                ui.label(egui::RichText::new(format!("Portföy: {}", self.portfolios.len())).small());
                ui.label(
                    egui::RichText::new(format!(
                        "Görev: {} açık",
                        self.tasks.iter().filter(|t| !t.done).count()
                    ))
                    .small(),
                );
            });
        if let Some(s) = target {
            if s == Screen::PortfolioForm {
                self.form = fresh_form();
            }
            self.navigate(s);
        }
    }

    fn messages(&mut self, ctx: &egui::Context) {
        let mut clear_toast = false;
        if let Some((msg, is_err, t)) = self.toast.clone() {
            if now_secs() - t > 4.0 {
                clear_toast = true;
            } else {
                let color = if is_err { egui::Color32::from_rgb(200, 60, 60) } else { egui::Color32::from_rgb(60, 160, 90) };
                egui::TopBottomPanel::bottom("toast").show(ctx, |ui| {
                    ui.colored_label(color, msg);
                });
            }
        }
        if clear_toast {
            self.toast = None;
        }

        if let Some(e) = self.error.clone() {
            egui::TopBottomPanel::bottom("err").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(egui::Color32::from_rgb(220, 70, 70), format!("Hata: {e}"));
                    if ui.small_button("Kapat").clicked() {
                        self.error = None;
                    }
                });
            });
        }
    }

    fn notification_popup(&mut self, ctx: &egui::Context) {
        if !self.show_notifications {
            return;
        }
        let mut open = self.show_notifications;
        egui::Window::new("Bildirimler")
            .open(&mut open)
            .default_width(380.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    if self.notifications.is_empty() {
                        ui.label("Bildirim yok.");
                    }
                    for n in &self.notifications {
                        let icon = match n.kind.as_str() {
                            "task" => "✅",
                            "goal" => "🎯",
                            _ => "🔔",
                        };
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.label(egui::RichText::new(format!("{icon} {}", n.title)).strong());
                            ui.label(&n.body);
                            ui.label(egui::RichText::new(&n.created_at).small().weak());
                        });
                    }
                });
            });
        self.show_notifications = open;
    }

    fn cover_zoom_window(&mut self, ctx: &egui::Context) {
        if let Some((name, _)) = self.cover_zoom.clone() {
            let mut open = true;
            let mut close = false;
            egui::Window::new("Resim")
                .open(&mut open)
                .default_size([700.0, 520.0])
                .show(ctx, |ui| {
                    if let Some(tex) = self.texture(ctx, &name) {
                        let size = tex.size_vec2();
                        let avail = ui.available_width();
                        let scale = (avail / size.x).min(480.0 / size.y).min(2.0);
                        ui.add(egui::Image::new(egui::load::SizedTexture::new(
                            tex.id(),
                            size * scale.max(0.1),
                        )));
                    } else {
                        ui.spinner();
                    }
                    if ui.button("Kapat").clicked() {
                        close = true;
                    }
                });
            if !open || close {
                self.cover_zoom = None;
            }
        }
    }
}

/// Renkli ikonlu, bold, sola yaslı menü düğmesi.
fn nav_button(
    ui: &mut egui::Ui,
    target: &mut Option<Screen>,
    current: Screen,
    to: Screen,
    icon: &str,
    label: &str,
    color: egui::Color32,
) {
    let selected = current == to;
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(w, 30.0), egui::Sense::click());
    let p = ui.painter_at(rect);
    let v = ui.visuals();
    let bg = if selected {
        v.selection.bg_fill
    } else if resp.hovered() {
        v.widgets.hovered.bg_fill
    } else {
        egui::Color32::TRANSPARENT
    };
    p.rect_filled(rect, 6.0, bg);
    p.text(
        egui::pos2(rect.left() + 15.0, rect.center().y),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(15.0),
        color,
    );
    p.text(
        egui::pos2(rect.left() + 33.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::new(13.5, egui::FontFamily::Name("navbold".into())),
        if selected { v.strong_text_color() } else { v.text_color() },
    );
    if resp.clicked() {
        *target = Some(to);
    }
}

/// Boşluk tuşuna basılınca ayraç ekler: "412 " -> "412/"
pub fn smart_separator(s: &str, sep: char) -> String {
    if s.ends_with(' ') {
        let trimmed = s.trim_end_matches(' ');
        if !trimmed.ends_with(sep) && !trimmed.is_empty() {
            return format!("{trimmed}{sep}");
        }
        return trimmed.to_string();
    }
    s.to_string()
}

fn tr_upper(c: char) -> char {
    match c {
        'i' => 'İ',
        'ı' => 'I',
        c => c.to_uppercase().next().unwrap_or(c),
    }
}

fn tr_lower(c: char) -> char {
    match c {
        'I' => 'ı',
        'İ' => 'i',
        c => c.to_lowercase().next().unwrap_or(c),
    }
}

/// Her kelimenin ilk harfi büyük (Türkçe uyumlu): "ali veli" -> "Ali Veli"
pub fn tr_title_case(s: &str) -> String {
    let mut out = String::new();
    let mut word_start = true;
    for c in s.chars() {
        if c.is_whitespace() {
            word_start = true;
            out.push(c);
        } else if word_start {
            out.push(tr_upper(c));
            word_start = false;
        } else {
            out.push(tr_lower(c));
        }
    }
    out
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ClientConfig {
    server_url: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    zoom: f64,
    #[serde(default)]
    font_family: usize,
    #[serde(default = "d_font_size")]
    font_size: f64,
    #[serde(default)]
    accent: String,
}

fn d_font_size() -> f64 { 14.0 }

fn config_file() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("client_config.json")
}

fn load_config() -> ClientConfig {
    std::fs::read_to_string(config_file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

impl App {
    // =====================================================================
    //  KARŞILAMA / GALERİ
    // =====================================================================
    fn welcome_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading("Portföyler");
            if ui.button("⟳ Yenile").clicked() {
                self.load_portfolios();
            }
            ui.separator();
            ui.label("Ara:");
            if ui.add(egui::TextEdit::singleline(&mut self.search).desired_width(180.0)).lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter))
            {
                self.load_portfolios();
            }
            if ui.button("Ara").clicked() {
                self.load_portfolios();
            }
        });
        ui.horizontal_wrapped(|ui| {
            ui.label("İlçe:");
            combo_index(ui, "w_ilce", &mut self.filter_ilce, &["Tümü"], common::mugla::DISTRICTS);
            ui.label("Tür:");
            combo_index(ui, "w_listing", &mut self.filter_listing, &["Tümü"], &["Satılık", "Kiralık"]);
            ui.label("Durum:");
            combo_index(
                ui,
                "w_status",
                &mut self.filter_status,
                &["Tümü"],
                &["Aktif", "Yayında", "Kiralandı", "Satıldı", "Taslak"],
            );
            if ui.button("Filtrele").clicked() {
                self.load_portfolios();
            }
            let prev = self.only_mine;
            ui.checkbox(&mut self.only_mine, "Sadece benim");
            if prev != self.only_mine {
                self.load_portfolios();
            }
            if self.is_admin() {
                let prevd = self.show_deleted;
                ui.checkbox(&mut self.show_deleted, "🗑 Silinmişleri göster");
                if prevd != self.show_deleted {
                    self.load_portfolios();
                }
            }
            ui.separator();
            ui.selectable_value(&mut self.list_view, false, "▦ Görsel");
            ui.selectable_value(&mut self.list_view, true, "☰ Liste");
        });
        ui.separator();

        if self.portfolios.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.label("Henüz portföy yok. 'Yeni Portföy' ile başlayın.");
            });
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.list_view {
                self.portfolio_list_view(ui, ctx);
            } else {
                self.portfolio_grid_view(ui, ctx);
            }
        });
    }

    /// Ctrl + fare tekerleği ile kart boyutlandırma (vitrin ekranında).
    fn handle_card_zoom(&mut self, ctx: &egui::Context) {
        if self.screen != Screen::Welcome {
            return;
        }
        let (ctrl, scroll_y) = ctx.input(|i| (i.modifiers.ctrl, i.smooth_scroll_delta.y));
        if ctrl && scroll_y != 0.0 {
            self.card_scale = (self.card_scale * (1.0 + scroll_y * 0.0012)).clamp(0.55, 2.5);
        }
    }

    fn portfolio_grid_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let card_w = 220.0 * self.card_scale;
        let cols = ((ui.available_width() / (card_w + 12.0)).floor() as usize).max(1);
        let mut idx = 0;
        while idx < self.portfolios.len() {
            ui.horizontal(|ui| {
                for _ in 0..cols {
                    if idx >= self.portfolios.len() {
                        break;
                    }
                    let p = self.portfolios[idx].clone();
                    self.portfolio_card(ui, ctx, &p, card_w);
                    idx += 1;
                }
            });
            ui.add_space(4.0);
        }
    }

    /// Kapak resmini en-boy oranını bozmadan (cover) kırparak çizer.
    fn draw_cover_image(painter: &egui::Painter, tex: &egui::TextureHandle, rect: egui::Rect) {
        let ts = tex.size_vec2();
        let a_img = ts.x / ts.y;
        let a_rect = rect.width() / rect.height();
        // Merkezden kırpma: kart hedef oranına uyan UV penceresi (cover)
        let uv = if a_img > a_rect {
            let cut = (1.0 - a_rect / a_img) * 0.5;
            egui::Rect::from_min_max(egui::pos2(cut, 0.0), egui::pos2(1.0 - cut, 1.0))
        } else {
            let cut = (1.0 - a_img / a_rect) * 0.5;
            egui::Rect::from_min_max(egui::pos2(0.0, cut), egui::pos2(1.0, 1.0 - cut))
        };
        painter.image(tex.id(), rect, uv, egui::Color32::WHITE);
    }

    fn portfolio_card(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, p: &Portfolio, w: f32) {
        let inner_w = w - 14.0;
        let img_h = (130.0 * self.card_scale).max(90.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(w);
            ui.set_min_size(egui::vec2(w, img_h + 96.0 * self.card_scale));
            ui.vertical(|ui| {
                ui.set_width(inner_w);
                let cover = p.images.first().cloned();
                let (rect, resp) = ui.allocate_exact_size(egui::vec2(inner_w, img_h), egui::Sense::click());
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, 4.0, egui::Color32::from_gray(60));
                if let Some(name) = &cover {
                    if let Some(tex) = self.texture(ctx, name) {
                        Self::draw_cover_image(&painter, &tex, rect);
                    }
                } else {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Resim yok",
                        egui::FontId::proportional(14.0 * self.card_scale),
                        egui::Color32::GRAY,
                    );
                }
                if resp.clicked() {
                    self.detail_id = Some(p.id);
                    self.navigate(Screen::PortfolioDetail);
                }
                ui.add_space(3.0);

                // Başlık: tek satır, taşma "..." ile kırpılır
                let mut title = egui::Label::new(egui::RichText::new(&p.title).strong().size(14.0 * self.card_scale));
                title = title.truncate();
                ui.add(title);
                // Tür/kategori ve konum: tek satır, kırpılır
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("{} • {} / {}", p.listing_type_label(), p.ilce, p.mahalle))
                        .small(),
                ).truncate());
                let price = if p.price > 0.0 {
                    format_price(p.price, &p.currency)
                } else {
                    "Fiyat yok".into()
                };
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(price)
                            .color(egui::Color32::from_rgb(46, 116, 181))
                            .strong()
                            .size(13.0 * self.card_scale),
                    ).truncate());
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(p.status_label()).small().weak(),
                    ).truncate());
                    if p.is_deleted {
                        ui.label(egui::RichText::new("🗑 silinmiş").small().color(egui::Color32::from_rgb(220, 80, 80)));
                    }
                });
            });
        });
    }

    fn portfolio_list_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        for p in self.portfolios.clone() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (rect, resp) = ui.allocate_exact_size(egui::vec2(90.0, 68.0), egui::Sense::click());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 4.0, egui::Color32::from_gray(60));
                    if let Some(name) = p.images.first() {
                        if let Some(tex) = self.texture(ctx, name) {
                            Self::draw_cover_image(&painter, &tex, rect);
                        }
                    }
                    if resp.clicked() {
                        self.detail_id = Some(p.id);
                        self.navigate(Screen::PortfolioDetail);
                    }
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&p.title).strong().size(15.0));
                        ui.label(format!(
                            "{} {} • {} / {} • {}",
                            p.listing_type_label(),
                            p.category_label(),
                            p.ilce,
                            p.mahalle,
                            p.status_label()
                        ));
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format_price(p.price, &p.currency))
                                    .color(egui::Color32::from_rgb(46, 116, 181))
                                    .strong(),
                            );
                            ui.label(egui::RichText::new(format!("Danışman: {}", p.created_by_name)).small().weak());
                            ui.label(egui::RichText::new(&p.created_at).small().weak());
                        });
                    });
                });
            });
        }
    }

    // =====================================================================
    //  PORTFÖY FORMU
    // =====================================================================
    fn portfolio_form_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let editing = self.form.edit_id.is_some();
        ui.heading(if editing { "Portföyü Düzenle" } else { "Yeni Portföy" });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("pf_grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
                ui.label("Başlık *");
                ui.add(egui::TextEdit::singleline(&mut self.form.title).desired_width(360.0));
                ui.end_row();

                ui.label("İlan Türü");
                combo_index(ui, "pf_lt", &mut self.form.listing_type, &[], &["Satılık", "Kiralık"]);
                ui.end_row();

                ui.label("Kategori");
                combo_index(ui, "pf_cat", &mut self.form.category, &[], &["Konut", "İş Yeri", "Arsa", "Tarla", "Turizm"]);
                ui.end_row();

                ui.label("Durum");
                combo_index(ui, "pf_st", &mut self.form.status, &[], &["Aktif", "Taslak", "Yayında", "Kiralandı", "Satıldı"]);
                ui.end_row();

                ui.label("Fiyat");
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let before = self.form.price_text.clone();
                        ui.add(
                            egui::TextEdit::singleline(&mut self.form.price_text)
                                .desired_width(150.0)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("6.000.000"),
                        );
                        if self.form.price_text != before {
                            self.form.price_text = format_price_input(&self.form.price_text);
                        }
                        combo_index(ui, "pf_cur", &mut self.form.currency, &[], &CURRENCIES);
                    });
                    // Taşınmaz m² fiyatı otomatik hesap
                    let brut = self.form.area_m2;
                    let price = format_price_input(&self.form.price_text)
                        .replace('.', "")
                        .parse::<f64>()
                        .unwrap_or(0.0);
                    if brut > 0.0 && price > 0.0 {
                        let m2 = price / brut;
                        ui.label(
                            egui::RichText::new(format!("Taşınmaz m² fiyatı: {:.0} {}/m²", m2, CURRENCIES[self.form.currency.min(3)]))
                                .small()
                                .color(egui::Color32::from_rgb(90, 160, 90)),
                        );
                    }
                });
                ui.end_row();

                ui.label("Alan (Brüt m²)");
                ui.add(egui::DragValue::new(&mut self.form.area_m2).speed(1.0).max_decimals(0));
                ui.end_row();

                ui.label("Alan (Net m²)");
                ui.add(egui::DragValue::new(&mut self.form.area_net).speed(1.0).max_decimals(0));
                ui.end_row();

                ui.label("Ada / Parsel");
                let before = self.form.ada_parsel.clone();
                ui.add(egui::TextEdit::singleline(&mut self.form.ada_parsel).desired_width(200.0).hint_text("412 7 → 412/7"));
                if self.form.ada_parsel != before {
                    self.form.ada_parsel = smart_separator(&self.form.ada_parsel, '/');
                }
                ui.end_row();

                ui.label("Oda Sayısı");
                let before = self.form.room_count.clone();
                ui.add(egui::TextEdit::singleline(&mut self.form.room_count).desired_width(140.0).hint_text("3 1 → 3+1"));
                if self.form.room_count != before {
                    self.form.room_count = smart_separator(&self.form.room_count, '+');
                }
                ui.end_row();

                ui.label("Bina Yaşı");
                ui.add(egui::TextEdit::singleline(&mut self.form.building_age).desired_width(140.0).hint_text("5"));
                ui.end_row();

                ui.label("Kat / Toplam Kat");
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.form.floor).desired_width(70.0));
                    ui.label("/");
                    ui.add(egui::TextEdit::singleline(&mut self.form.total_floors).desired_width(70.0));
                });
                ui.end_row();

                ui.label("Isıtma (çoklu seçim)");
                ui.horizontal_wrapped(|ui| {
                    for (i, h) in HEATINGS.iter().enumerate() {
                        let mut sel = self.form.heating_choices.contains(&i);
                        if ui.checkbox(&mut sel, *h).changed() {
                            if sel {
                                if !self.form.heating_choices.contains(&i) {
                                    self.form.heating_choices.push(i);
                                }
                            } else {
                                self.form.heating_choices.retain(|x| *x != i);
                            }
                        }
                    }
                });
                ui.end_row();

                ui.label("Eşyalı");
                ui.checkbox(&mut self.form.furnished, "Eşyalı");
                ui.end_row();

                ui.label("İl");
                ui.horizontal(|ui| {
                    if self.form.other_il {
                        ui.add(egui::TextEdit::singleline(&mut self.form.il_custom).desired_width(140.0));
                    } else {
                        ui.label(egui::RichText::new("Muğla").strong());
                    }
                    let mut t = self.form.other_il;
                    if ui.checkbox(&mut t, "Farklı il gireceğim").changed() {
                        self.form.other_il = t;
                        if t && self.form.il_custom.is_empty() {
                            self.form.il_custom = "Muğla".to_string();
                        }
                    }
                });
                ui.end_row();

                ui.label("İlçe *");
                combo_index(ui, "pf_ilce", &mut self.form.ilce, &["Seçiniz"], common::mugla::DISTRICTS);
                ui.end_row();

                ui.label("Mahalle *");
                let mahs = self.form.current_mahalles();
                combo_index(ui, "pf_mah", &mut self.form.mahalle, &["Seçiniz"], &mahs);
                ui.end_row();

                ui.label("Açık Adres");
                let before = self.form.address.clone();
                ui.add(egui::TextEdit::singleline(&mut self.form.address).desired_width(360.0));
                if self.form.address != before {
                    self.form.address = tr_title_case(&self.form.address);
                }
                ui.end_row();

                ui.label("Konum (Enlem, Boylam)");
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut self.form.latitude).speed(0.0001).max_decimals(6).prefix("En: "));
                        ui.add(egui::DragValue::new(&mut self.form.longitude).speed(0.0001).max_decimals(6).prefix("Boy: "));
                    });
                    ui.horizontal_wrapped(|ui| {
                        let ilce = if self.form.ilce == 0 { "Fethiye".to_string() } else { common::mugla::DISTRICTS[self.form.ilce - 1].to_string() };
                        let mah = { let m = self.form.current_mahalles(); if self.form.mahalle == 0 || self.form.mahalle > m.len() { String::new() } else { m[self.form.mahalle - 1].to_string() } };
                        if ui.small_button("🗺 Google Uydu (seç)").clicked() {
                            let (lat, lng) = self.picked_coord_hint();
                            let _ = open::that(format!("https://www.google.com/maps/@{lat},{lng},18z/data=!3m1!1e3"));
                            self.set_error("Uydu haritada yere sağ tıklayıp koordinatı kopyalayın, aşağıya yapıştırıp Uygula'ya basın.".to_string());
                        }
                        if ui.small_button("🛰 Yandex Uydu (seç)").clicked() {
                            let (lat, lng) = self.picked_coord_hint();
                            let _ = open::that(format!("https://yandex.com.tr/maps/?l=sat&ll={lng},{lat}&z=18"));
                            self.set_error("Haritada yere sağ tıklayıp koordinatı kopyalayın, aşağıya yapıştırıp Uygula'ya basın.".to_string());
                        }
                        if !mah.is_empty() && ui.small_button("🏘 Mahalleyi Haritada Ara").clicked() {
                            let q = format!("{} {} Muğla", ilce, mah);
                            let _ = open::that(format!("https://www.google.com/maps/search/?api=1&query={}", urlencode(&q)));
                        }
                    });
                    ui.horizontal(|ui| {
                        let before = self.form.coord_paste.clone();
                        ui.add(egui::TextEdit::singleline(&mut self.form.coord_paste).desired_width(190.0).hint_text("Koordinatı buraya yapıştırın"));
                        if self.form.coord_paste != before {
                            if let Some((lat, lng)) = parse_coords(&self.form.coord_paste) {
                                self.form.latitude = lat;
                                self.form.longitude = lng;
                            }
                        }
                        if ui.small_button("Uygula").clicked() {
                            if let Some((lat, lng)) = parse_coords(&self.form.coord_paste) {
                                self.form.latitude = lat;
                                self.form.longitude = lng;
                                self.form.coord_paste.clear();
                                self.toast("Koordinat uygulandı", false);
                            } else {
                                self.set_error("Koordinat okunamadı. Örnek: 36.6214, 29.1163".into());
                            }
                        }
                    });
                });
                ui.end_row();

                ui.label("Açıklama");
                ui.add(
                    egui::TextEdit::multiline(&mut self.form.description)
                        .desired_width(360.0)
                        .desired_rows(5),
                );
                ui.end_row();
            });

            ui.add_space(10.0);
            ui.separator();
            ui.label(egui::RichText::new("Taşınmaz Sahibi (CRM)").strong());
            ui.label(
                egui::RichText::new("Bu bilgiler CRM kaydı olarak saklanır; CRM listesini yalnızca yönetici görür.")
                    .small()
                    .weak(),
            );
            let mut owner_options: Vec<String> = vec!["Kaydetme".into(), "Yeni sahip ekle".into()];
            if self.is_admin() {
                for c in &self.crm {
                    owner_options.push(format!("#{} {}", c.id, c.full_name));
                }
            }
            egui::ComboBox::from_label("Sahip Kaydı")
                .selected_text(owner_options.get(self.form.owner_choice).cloned().unwrap_or_default())
                .show_ui(ui, |ui| {
                    for (i, o) in owner_options.iter().enumerate() {
                        ui.selectable_value(&mut self.form.owner_choice, i, o);
                    }
                });
            if self.form.owner_choice == 1 {
                egui::Grid::new("owner_grid").num_columns(2).show(ui, |ui| {
                    ui.label("Ad Soyad *");
                    ui.add(egui::TextEdit::singleline(&mut self.form.new_owner_name).desired_width(300.0));
                    ui.end_row();
                    ui.label("Telefon");
                    ui.add(egui::TextEdit::singleline(&mut self.form.new_owner_phone).desired_width(300.0));
                    ui.end_row();
                    ui.label("E-posta");
                    ui.add(egui::TextEdit::singleline(&mut self.form.new_owner_email).desired_width(300.0));
                    ui.end_row();
                    ui.label("Tip");
                    combo_index(ui, "owner_type", &mut self.form.owner_type_choice, &[], &OWNER_TYPES);
                    ui.end_row();
                    ui.label("Notlar");
                    ui.add(egui::TextEdit::multiline(&mut self.form.new_owner_notes).desired_width(300.0).desired_rows(2));
                    ui.end_row();
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.label(egui::RichText::new("Resimler").strong());
            ui.label(egui::RichText::new("Not: Resim dosyalarını pencereye sürükleyip bırakabilirsiniz.").small().weak());

            if !self.form.existing_images.is_empty() {
                ui.label("Mevcut resimler (ilk resim kapak olur):");
                let imgs = self.form.existing_images.clone();
                let mut remove: Option<String> = None;
                ui.horizontal_wrapped(|ui| {
                    for (i, name) in imgs.iter().enumerate() {
                        ui.vertical(|ui| {
                            if let Some(tex) = self.texture(ctx, name) {
                                ui.add(egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(90.0, 70.0))));
                            }
                            if i == 0 {
                                ui.label(egui::RichText::new("Kapak").small().strong());
                            }
                            if ui.small_button("Sil").clicked() {
                                remove = Some(name.clone());
                            }
                        });
                    }
                });
                if let Some(name) = remove {
                    if let Some(id) = self.form.edit_id {
                        match self.api.delete_portfolio_image(id, &name) {
                            Ok(_) => {
                                self.form.existing_images.retain(|x| x != &name);
                                self.textures.remove(&name);
                            }
                            Err(e) => self.set_error(e),
                        }
                    }
                }
            }

            ui.horizontal(|ui| {
                if ui.button("＋ Resim Ekle (dosya seç)").clicked() {
                    if let Some(files) = pick_files() {
                        self.form.new_images.extend(files);
                    }
                }
                if !self.form.new_images.is_empty() && ui.button("Listeyi temizle").clicked() {
                    self.form.new_images.clear();
                }
            });
            if !self.form.new_images.is_empty() {
                ui.label(format!("{} yeni resim seçildi:", self.form.new_images.len()));
                for p in &self.form.new_images {
                    ui.label(egui::RichText::new(format!("• {}", p.display())).small());
                }
            }

            ui.add_space(14.0);
            ui.horizontal(|ui| {
                let save_label = if editing { "Güncelle" } else { "Kaydet" };
                if ui
                    .add(egui::Button::new(egui::RichText::new(save_label).size(16.0)).min_size(egui::vec2(140.0, 36.0)))
                    .clicked()
                {
                    self.save_portfolio();
                }
                if ui.button("İptal").clicked() {
                    self.form = fresh_form();
                    self.navigate(Screen::Welcome);
                }
            });
        });
    }

    fn digit_count(s: &str) -> usize {
        s.chars().filter(|c| c.is_ascii_digit()).count()
    }

    /// Ayarlarda tanımlı kurallara göre formu doğrular; hata mesajı döndürür.
    fn validate_portfolio_form(&self) -> Option<String> {
        let r = &self.settings;
        let title_len = self.form.title.trim().chars().count() as i64;
        if title_len < r.val_title_min.max(1) {
            return Some(format!(
                "Başlık en az {} karakter olmalıdır",
                r.val_title_min.max(1)
            ));
        }
        if r.val_desc_min > 0 && (self.form.description.trim().chars().count() as i64) < r.val_desc_min {
            return Some(format!(
                "Açıklama en az {} karakter olmalıdır",
                r.val_desc_min
            ));
        }
        if self.form.price < r.val_price_min {
            return Some(format!("Fiyat en az {} olmalıdır", r.val_price_min));
        }
        if self.form.area_m2 < r.val_area_min {
            return Some(format!("Alan en az {} m² olmalıdır", r.val_area_min));
        }
        if self.form.owner_choice == 1 {
            if r.val_phone_required && self.form.new_owner_phone.trim().is_empty() {
                return Some("Taşınmaz sahibi telefonu zorunludur".to_string());
            }
            if r.val_phone_digits > 0
                && !self.form.new_owner_phone.trim().is_empty()
                && (Self::digit_count(&self.form.new_owner_phone) as i64) < r.val_phone_digits
            {
                return Some(format!(
                    "Telefon en az {} rakam içermelidir",
                    r.val_phone_digits
                ));
            }
            if r.val_email_required && self.form.new_owner_email.trim().is_empty() {
                return Some("Taşınmaz sahibi e-posta adresi zorunludur".to_string());
            }
            if !self.form.new_owner_email.trim().is_empty() && !self.form.new_owner_email.contains('@') {
                return Some("Geçerli bir e-posta adresi giriniz".to_string());
            }
        }
        None
    }

    fn save_portfolio(&mut self) {
        if let Some(msg) = self.validate_portfolio_form() {
            self.set_error(msg);
            return;
        }
        let ilce = if self.form.ilce == 0 {
            String::new()
        } else {
            common::mugla::DISTRICTS[self.form.ilce - 1].to_string()
        };
        let mahalle = {
            let mahs = self.form.current_mahalles();
            if self.form.mahalle == 0 || self.form.mahalle > mahs.len() {
                String::new()
            } else {
                mahs[self.form.mahalle - 1].to_string()
            }
        };

        // Sahip kaydı
        let mut owner_id: Option<i64> = None;
        if self.form.owner_choice == 1 {
            if self.form.new_owner_name.trim().is_empty() {
                self.set_error("Yeni sahip için Ad Soyad zorunludur".into());
                return;
            }
            let crm_in = CrmInput {
                full_name: self.form.new_owner_name.clone(),
                phone: self.form.new_owner_phone.clone(),
                email: self.form.new_owner_email.clone(),
                address: self.form.address.clone(),
                owner_type: OWNER_TYPES[self.form.owner_type_choice.min(OWNER_TYPES.len() - 1)].to_string(),
                notes: self.form.new_owner_notes.clone(),
            };
            match self.api.create_crm(&crm_in) {
                Ok(c) => owner_id = Some(c.id),
                Err(e) => {
                    self.set_error(e);
                    return;
                }
            }
        } else if self.form.owner_choice >= 2 && self.is_admin() {
            if let Some(c) = self.crm.get(self.form.owner_choice - 2) {
                owner_id = Some(c.id);
            }
        }

        let price_val = self
            .form
            .price_text
            .replace('.', "")
            .trim()
            .parse::<f64>()
            .unwrap_or(self.form.price);
        let input = PortfolioInput {
            title: self.form.title.trim().to_string(),
            listing_type: ListingType::ALL[self.form.listing_type.min(1)].as_str().to_string(),
            category: PropertyCategory::ALL[self.form.category.min(4)].as_str().to_string(),
            status: PortfolioStatus::ALL[self.form.status.min(4)].as_str().to_string(),
            price: price_val,
            currency: CURRENCIES[self.form.currency.min(3)].to_string(),
            area_m2: self.form.area_m2,
            area_net: self.form.area_net,
            ada_parsel: self.form.ada_parsel.clone(),
            room_count: self.form.room_count.clone(),
            building_age: self.form.building_age.clone(),
            floor: self.form.floor.clone(),
            total_floors: self.form.total_floors.clone(),
            heating: {
                let mut names: Vec<&str> = self
                    .form
                    .heating_choices
                    .iter()
                    .filter_map(|i| HEATINGS.get(*i).copied())
                    .collect();
                names.sort();
                names.join(", ")
            },
            furnished: self.form.furnished,
            il: if self.form.other_il && !self.form.il_custom.trim().is_empty() {
                self.form.il_custom.trim().to_string()
            } else {
                "Muğla".to_string()
            },
            ilce,
            mahalle,
            address: self.form.address.clone(),
            latitude: self.form.latitude,
            longitude: self.form.longitude,
            description: self.form.description.clone(),
            owner_crm_id: owner_id,
        };

        let result = match self.form.edit_id {
            Some(id) => self.api.update_portfolio(id, &input),
            None => self.api.create_portfolio(&input),
        };

        match result {
            Ok(p) => {
                if !self.form.new_images.is_empty() {
                    match self.api.upload_portfolio_images(p.id, &self.form.new_images) {
                        Ok(names) => {
                            for n in names {
                                self.textures.remove(&n);
                            }
                        }
                        Err(e) => self.set_error(e),
                    }
                }
                self.form = fresh_form();
                self.toast("Portföy kaydedildi", false);
                self.navigate(Screen::Welcome);
            }
            Err(e) => self.set_error(e),
        }
    }

    // =====================================================================
    //  PORTFÖY DETAY
    // =====================================================================
    fn detail_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let id = match self.detail_id {
            Some(id) => id,
            None => {
                ui.label("Portföy seçilmedi");
                return;
            }
        };
        let portfolio = match self.api.get_portfolio(id) {
            Ok(p) => p,
            Err(e) => {
                ui.label(format!("Portföy yüklenemedi: {e}"));
                if ui.button("Geri").clicked() {
                    self.navigate(Screen::Welcome);
                }
                return;
            }
        };

        ui.horizontal(|ui| {
            if ui.button("← Geri").clicked() {
                self.navigate(Screen::Welcome);
            }
            ui.heading(&portfolio.title);
        });
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if !portfolio.images.is_empty() {
                let imgs = portfolio.images.clone();
                ui.horizontal_wrapped(|ui| {
                    for name in &imgs {
                        match self.texture(ctx, name) {
                            Some(tex) => {
                                let resp = ui.add(
                                    egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(180.0, 135.0)))
                                        .sense(egui::Sense::click()),
                                );
                                if resp.clicked() {
                                    self.cover_zoom = Some((name.clone(), 1.0));
                                }
                            }
                            None => {
                                // Resim indirilemedi/çözülemedi: boş alan yerine net bilgi + yeniden dene
                                let (rect, _) = ui.allocate_exact_size(egui::vec2(180.0, 135.0), egui::Sense::hover());
                                ui.painter_at(rect).rect_filled(rect, 4.0, egui::Color32::from_gray(55));
                                ui.painter_at(rect).text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("Resim yüklenemedi\n{}", name),
                                    egui::FontId::proportional(11.0),
                                    egui::Color32::from_rgb(230, 120, 120),
                                );
                                if ui.small_button("⟳ Yeniden Dene").clicked() {
                                    self.textures.remove(name);
                                    self.set_error(format!(
                                        "Resim sunucudan indirilemedi: {name}. Sunucunun data/uploads klasörünü ve bağlantıyı kontrol edin."
                                    ));
                                }
                            }
                        }
                    }
                });
                ui.add_space(8.0);
            }

            egui::Grid::new("detail_grid").num_columns(2).spacing([20.0, 6.0]).show(ui, |ui| {
                detail_row(ui, "Tür / Kategori", &format!("{} / {}", portfolio.listing_type_label(), portfolio.category_label()));
                detail_row(ui, "Durum", &portfolio.status_label());
                detail_row(ui, "Fiyat", &format_price(portfolio.price, &portfolio.currency));
                detail_row(ui, "Alan", &format!("{} m² brüt / {} m² net", portfolio.area_m2, portfolio.area_net));
                if !portfolio.ada_parsel.is_empty() {
                    detail_row(ui, "Ada / Parsel", &portfolio.ada_parsel);
                }
                if portfolio.area_m2 > 0.0 && portfolio.price > 0.0 {
                    detail_row(ui, "m² Fiyatı", &format!("{:.0} {}/m²", portfolio.price / portfolio.area_m2, portfolio.currency));
                }
                detail_row(ui, "Oda", &portfolio.room_count);
                detail_row(ui, "Bina Yaşı", &portfolio.building_age);
                detail_row(ui, "Kat", &format!("{} / {}", portfolio.floor, portfolio.total_floors));
                detail_row(ui, "Isıtma", &portfolio.heating);
                detail_row(ui, "Eşyalı", if portfolio.furnished { "Evet" } else { "Hayır" });
                detail_row(ui, "Konum", &format!("{} / {} / {}", portfolio.il, portfolio.ilce, portfolio.mahalle));
                detail_row(ui, "Adres", &portfolio.address);
                if portfolio.latitude != 0.0 || portfolio.longitude != 0.0 {
                    ui.label(egui::RichText::new("Koordinat").strong());
                    let url = format!(
                        "https://www.google.com/maps?q={},{}",
                        portfolio.latitude, portfolio.longitude
                    );
                    if ui
                        .link(
                            egui::RichText::new(format!(
                                "🗺 {:.5}, {:.5}   (tıkla → haritada aç)",
                                portfolio.latitude, portfolio.longitude
                            ))
                            .color(egui::Color32::from_rgb(86, 156, 214)),
                        )
                        .clicked()
                    {
                        let _ = open::that(url);
                    }
                    ui.end_row();
                }
                detail_row(ui, "Danışman", &portfolio.created_by_name);
                detail_row(ui, "Kayıt", &portfolio.created_at);
            });

            if !portfolio.description.is_empty() {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Açıklama").strong());
                ui.label(&portfolio.description);
            }

            ui.add_space(12.0);
            ui.separator();
            ui.label(egui::RichText::new("Çıktı / Paylaşım").strong());
            ui.horizontal_wrapped(|ui| {
                if ui.button("📄 PDF Oluştur").clicked() {
                    self.export_portfolio(&portfolio, "pdf");
                }
                if ui.button("📝 DOCX Oluştur").clicked() {
                    self.export_portfolio(&portfolio, "docx");
                }
                if ui.button("📱 WhatsApp ile Paylaş").clicked() {
                    self.share_whatsapp(&portfolio);
                }
                if ui.button("📂 Çıktı Klasörünü Aç").clicked() {
                    let _ = open::that(export_dir());
                }
                if portfolio.latitude != 0.0 || portfolio.longitude != 0.0 {
                    if ui.button("🗺 Haritada Aç").clicked() {
                        let url = format!("https://www.google.com/maps?q={},{}", portfolio.latitude, portfolio.longitude);
                        let _ = open::that(url);
                    }
                }
            });

            ui.add_space(12.0);
            ui.separator();
            let owns = self.user.as_ref().map(|u| u.id == portfolio.created_by).unwrap_or(false);
            if owns || self.is_admin() {
                ui.horizontal(|ui| {
                    if ui.button("✏ Düzenle").clicked() {
                        self.start_edit(&portfolio);
                    }
                    if ui.button("🗑 Sil").clicked() {
                        match self.api.delete_portfolio(portfolio.id) {
                            Ok(_) => {
                                self.toast("Portföy silme komutu alındı (yönetici bilgilendirildi)", false);
                                self.navigate(Screen::Welcome);
                            }
                            Err(e) => self.set_error(e),
                        }
                    }
                    if self.is_admin() && portfolio.is_deleted && ui.button("♻ Geri Yükle").clicked() {
                        match self.api.restore_portfolio(portfolio.id) {
                            Ok(_) => {
                                self.toast("Portföy geri yüklendi", false);
                                self.navigate(Screen::Welcome);
                            }
                            Err(e) => self.set_error(e),
                        }
                    }
                });
            }
        });
    }

    /// Haritayı açarken kullanılacak başlangıç koordinatı (boşsa Fethiye merkezi).
    fn picked_coord_hint(&self) -> (f64, f64) {
        if self.form.latitude != 0.0 || self.form.longitude != 0.0 {
            (self.form.latitude, self.form.longitude)
        } else {
            (36.6214, 29.1163) // Fethiye merkez
        }
    }

    fn start_edit(&mut self, p: &Portfolio) {
        let ilce = common::mugla::DISTRICTS.iter().position(|d| *d == p.ilce).map(|i| i + 1).unwrap_or(0);
        let mahs = common::mugla::neighborhoods(&p.ilce);
        let mahalle = mahs.iter().position(|m| *m == p.mahalle).map(|i| i + 1).unwrap_or(0);
        let listing = match p.listing_type.as_str() { "kiralik" => 1, _ => 0 };
        let category = match p.category.as_str() {
            "isyeri" => 1, "arsa" => 2, "tarla" => 3, "turizm" => 4, _ => 0,
        };
        let status = match p.status.as_str() {
            "taslak" => 1, "yayinda" => 2, "kiralandi" => 3, "satildi" => 4, _ => 0,
        };
        let currency = CURRENCIES.iter().position(|c| *c == p.currency).unwrap_or(0);
        let heating_choices: Vec<usize> = p
            .heating
            .split(',')
            .map(|x| x.trim())
            .filter_map(|x| HEATINGS.iter().position(|h| *h == x))
            .collect();
        let owner_choice = if p.owner_crm_id.is_some() && self.is_admin() {
            self.crm.iter().position(|c| Some(c.id) == p.owner_crm_id).map(|i| i + 2).unwrap_or(0)
        } else {
            0
        };
        self.form = FormState {
            edit_id: Some(p.id),
            other_il: p.il != "Muğla",
            il_custom: p.il.clone(),
            title: p.title.clone(),
            listing_type: listing,
            category,
            status,
            price: p.price,
            currency,
            price_text: format_price_input(&format!("{}", p.price.round() as i64)),
            area_m2: p.area_m2,
            area_net: p.area_net,
            ada_parsel: p.ada_parsel.clone(),
            room_count: p.room_count.clone(),
            building_age: p.building_age.clone(),
            floor: p.floor.clone(),
            total_floors: p.total_floors.clone(),
            heating_choices,
            owner_type_choice: 0,
            furnished: p.furnished,
            ilce,
            mahalle,
            address: p.address.clone(),
            latitude: p.latitude,
            longitude: p.longitude,
            description: p.description.clone(),
            owner_choice,
            existing_images: p.images.clone(),
            ..Default::default()
        };
        self.navigate(Screen::PortfolioForm);
    }

    fn export_portfolio(&mut self, p: &Portfolio, kind: &str) {
        let dir = export_dir();
        let _ = std::fs::create_dir_all(&dir);
        let safe: String = p.title.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
        let agent = match &self.user {
            Some(u) => u.clone(),
            None => return,
        };
        // Sadece admin CRM görebildiği için sahip bilgisi varsa admin ise iliştirilir.
        let owner = if self.is_admin() {
            p.owner_crm_id.and_then(|id| self.crm.iter().find(|c| c.id == id).cloned())
        } else {
            None
        };
        let result: Result<PathBuf, String> = if kind == "pdf" {
            let imgs: Vec<Vec<u8>> = p.images.iter().filter_map(|n| self.api.download_file(n).ok()).collect();
            export::export_pdf(&dir.join(format!("{safe}.pdf")), p, owner.as_ref(), &agent, &self.settings, &imgs)
        } else {
            let imgs: Vec<Vec<u8>> = p.images.iter().filter_map(|n| self.api.download_file(n).ok()).collect();
            export::export_docx(
                &dir.join(format!("{safe}.docx")),
                p,
                owner.as_ref(),
                &agent,
                &self.settings,
                &imgs,
            )
        };
        match result {
            Ok(path) => {
                self.toast(format!("Oluşturuldu: {}", path.display()), false);
                let _ = open::that(&path);
            }
            Err(e) => self.set_error(format!("Çıktı üretilemedi: {e}")),
        }
    }

    fn share_whatsapp(&mut self, p: &Portfolio) {
        let text = format!(
            "{} - {} {} \n{} {} / {} \nFiyat: {} \n{}",
            self.settings.header_text,
            p.listing_type_label(),
            p.category_label(),
            p.il,
            p.ilce,
            p.mahalle,
            format_price(p.price, &p.currency),
            p.description
        );
        let url = format!("https://wa.me/?text={}", urlencode(&text));
        if let Err(e) = open::that(url) {
            self.set_error(format!("WhatsApp açılamadı: {e}"));
        }
    }
}

// =====================================================================
//  GÖREVLER / HEDEFLER / MESAİ
// =====================================================================

impl App {
    fn tasks_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Görevler ve Sorumluluklar");
        ui.separator();
        let mut refresh = false;

        egui::CollapsingHeader::new("➕ Yeni Görev / Not Ata")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("task_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label("Başlık *");
                    ui.add(egui::TextEdit::singleline(&mut self.task_form.title).desired_width(320.0));
                    ui.end_row();
                    ui.label("Açıklama");
                    ui.add(egui::TextEdit::multiline(&mut self.task_form.description).desired_width(320.0).desired_rows(2));
                    ui.end_row();
                    ui.label("Atanan Kişiler (çoklu seçim)");
                    // Çoklu seçim: her personel için onay kutusu; hiçbiri seçilmezse tüm ekip
                    if self.users.is_empty() {
                        ui.label(egui::RichText::new("Kullanıcı listesi yükleniyor...").small().weak());
                    } else {
                        ui.vertical(|ui| {
                            for u in self.users.clone() {
                                let name = if u.full_name.is_empty() { u.username.clone() } else { u.full_name.clone() };
                                let mut selected = self.task_assignees_sel.contains(&u.id);
                                if ui.checkbox(&mut selected, format!("{}  ({})", name, u.username)).changed() {
                                    if selected {
                                        if !self.task_assignees_sel.contains(&u.id) {
                                            self.task_assignees_sel.push(u.id);
                                        }
                                    } else {
                                        self.task_assignees_sel.retain(|id| *id != u.id);
                                    }
                                }
                            }
                        });
                        ui.label(
                            egui::RichText::new(format!(
                                "{} kişi seçildi • hiçbiri seçilmezse tüm ekibe gider",
                                self.task_assignees_sel.len()
                            ))
                            .small()
                            .weak(),
                        );
                    }
                    ui.end_row();
                    ui.label("Tarih / Saat");
                    ui.horizontal(|ui| {
                        Self::date_field(ui, &mut self.task_form.due_date, "GG-AA-YYYY");
                        ui.add(egui::TextEdit::singleline(&mut self.task_form.due_time).desired_width(80.0).hint_text("16:00"));
                    });
                    ui.end_row();
                    ui.label("Öncelik");
                    combo_index(ui, "task_prio", &mut self.task_form.priority, &[], &PRIORITIES);
                    ui.end_row();
                });
                if ui.button("Görevi Ata").clicked() {
                    let iso = if self.task_form.due_date.trim().is_empty() {
                        Some(String::new())
                    } else {
                        Self::iso_from_display(&self.task_form.due_date)
                    };
                    let iso = match iso {
                        Some(v) => v,
                        None => {
                            self.set_error("Geçerli bir tarih giriniz (GG-AA-YYYY)".into());
                            return;
                        }
                    };
                    let due = if iso.is_empty() {
                        String::new()
                    } else if self.task_form.due_time.trim().is_empty() {
                        iso
                    } else {
                        format!("{} {}", iso, self.task_form.due_time)
                    };
                    let input = TaskInput {
                        title: self.task_form.title.clone(),
                        description: self.task_form.description.clone(),
                        assigned_to: self.task_assignees_sel.clone(),
                        due_at: due,
                        priority: PRIORITIES[self.task_form.priority.min(2)].to_string(),
                    };
                    match self.api.create_task(&input) {
                        Ok(_) => {
                            self.toast("Görev atandı ve bildirimler gönderildi", false);
                            self.task_form = TaskForm::default();
                            self.task_assignees_sel.clear();
                            refresh = true;
                        }
                        Err(e) => self.set_error(e),
                    }
                }
            });

        ui.separator();
        let mut toggle: Option<i64> = None;
        let mut del: Option<i64> = None;
        for t in self.tasks.clone() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut done = t.done;
                    if ui.checkbox(&mut done, "").changed() {
                        toggle = Some(t.id);
                    }
                    ui.vertical(|ui| {
                        let title = if t.done {
                            egui::RichText::new(&t.title).strikethrough()
                        } else {
                            egui::RichText::new(&t.title).strong()
                        };
                        ui.label(title);
                        ui.label(egui::RichText::new(&t.description).small());
                        ui.horizontal(|ui| {
                            let who = if t.assigned_to.is_none() { "Tüm Ekip".to_string() } else { t.assigned_to_name.clone() };
                            ui.label(egui::RichText::new(format!("Hedef: {who}")).small().weak());
                            if !t.due_at.is_empty() {
                                ui.label(
                                    egui::RichText::new(format!("⏰ {}", t.due_at))
                                        .small()
                                        .color(egui::Color32::from_rgb(200, 120, 40)),
                                );
                            }
                            ui.label(egui::RichText::new(format!("Öncelik: {}", t.priority)).small().weak());
                            ui.label(egui::RichText::new(format!("Veren: {}", t.assigned_by_name)).small().weak());
                        });
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let owns = self.user.as_ref().map(|u| u.id == t.assigned_by).unwrap_or(false);
                        if owns || self.is_admin() {
                            if ui.small_button("Sil").clicked() {
                                del = Some(t.id);
                            }
                        }
                    });
                });
            });
        }
        if let Some(id) = toggle {
            let _ = self.api.toggle_task(id);
            refresh = true;
        }
        if let Some(id) = del {
            let _ = self.api.delete_task(id);
            refresh = true;
        }
        if refresh {
            self.load_tasks();
        }
    }

    fn goals_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Hedefler");
        ui.separator();
        let mut refresh = false;

        if self.is_admin() {
            egui::CollapsingHeader::new("➕ Yeni Hedef Ata")
                .default_open(false)
                .show(ui, |ui| {
                    egui::Grid::new("goal_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.label("Başlık *");
                        ui.add(egui::TextEdit::singleline(&mut self.goal_form.title).desired_width(320.0).hint_text("Aylık portföy hedefi"));
                        ui.end_row();
                        ui.label("Kriter");
                        combo_index(ui, "goal_metric", &mut self.goal_form.metric, &[], &METRICS);
                        ui.end_row();
                        ui.label("Hedef Sayı");
                        ui.add(egui::DragValue::new(&mut self.goal_form.target_count).range(0..=1000));
                        ui.end_row();
                        ui.label("Kapsam");
                        let mut opts = vec!["Tüm Ekip".to_string()];
                        for u in &self.users {
                            opts.push(if u.full_name.is_empty() { u.username.clone() } else { u.full_name.clone() });
                        }
                        combo_index(ui, "goal_scope", &mut self.goal_form.scope_user, &[], &opts.iter().map(|s| s.as_str()).collect::<Vec<_>>());
                        ui.end_row();
                        ui.label("Başlangıç / Bitiş");
                        ui.horizontal(|ui| {
                            Self::date_field(ui, &mut self.goal_form.period_start, "GG-AA-YYYY");
                            ui.label("—");
                            Self::date_field(ui, &mut self.goal_form.period_end, "GG-AA-YYYY");
                        });
                        ui.end_row();
                    });
                    if ui.button("Hedefi Ata").clicked() {
                        let ps = if self.goal_form.period_start.trim().is_empty() {
                            Some(String::new())
                        } else {
                            Self::iso_from_display(&self.goal_form.period_start)
                        };
                        let pe = if self.goal_form.period_end.trim().is_empty() {
                            Some(String::new())
                        } else {
                            Self::iso_from_display(&self.goal_form.period_end)
                        };
                        let (ps, pe) = match (ps, pe) {
                            (Some(a), Some(b)) => (a, b),
                            _ => {
                                self.set_error("Geçerli dönem tarihleri giriniz (GG-AA-YYYY)".into());
                                return;
                            }
                        };
                        let (scope, user_id) = if self.goal_form.scope_user == 0 {
                            ("team".to_string(), None)
                        } else {
                            ("user".to_string(), self.users.get(self.goal_form.scope_user - 1).map(|u| u.id))
                        };
                        let input = GoalInput {
                            scope,
                            user_id,
                            title: self.goal_form.title.clone(),
                            metric: METRICS[self.goal_form.metric.min(4)].to_string(),
                            target_count: self.goal_form.target_count,
                            period_start: ps,
                            period_end: pe,
                        };
                        match self.api.create_goal(&input) {
                            Ok(_) => {
                                self.toast("Hedef atandı", false);
                                self.goal_form = GoalForm::default();
                                refresh = true;
                            }
                            Err(e) => self.set_error(e),
                        }
                    }
                });
            ui.separator();
        }

        let mut del: Option<i64> = None;
        for g in self.goals.clone() {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(&g.title).strong());
                        let scope = if g.scope == "team" { "Tüm Ekip".to_string() } else { g.user_name.clone() };
                        ui.label(egui::RichText::new(format!("Kapsam: {scope} • Kriter: {}", g.metric)).small());
                        ui.label(egui::RichText::new(format!("Dönem: {} - {}", g.period_start, g.period_end)).small().weak());
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.is_admin() && ui.small_button("Sil").clicked() {
                            del = Some(g.id);
                        }
                        let progress = if g.target_count > 0 {
                            (g.current_count as f32 / g.target_count as f32).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        ui.vertical(|ui| {
                            ui.label(format!("{}/{}", g.current_count, g.target_count));
                            ui.add(egui::ProgressBar::new(progress).desired_width(180.0));
                        });
                    });
                });
            });
        }
        if let Some(id) = del {
            let _ = self.api.delete_goal(id);
            refresh = true;
        }
        if refresh {
            self.load_goals();
        }
    }

    fn shifts_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Vardiya ve Mesai");
        ui.separator();

        let tabs = [
            ("📋 Vardiya Takvimi", 0usize),
            ("✅ Mesai Onay Listesi", 1usize),
            ("📈 Vardiya Raporu", 2usize),
        ];
        ui.horizontal(|ui| {
            for (label, i) in tabs {
                let admin_only = i > 0;
                if admin_only && !self.is_admin() {
                    continue;
                }
                if ui.selectable_label(self.shift_tab == i, label).clicked() {
                    self.shift_tab = i;
                }
            }
        });
        ui.separator();

        match self.shift_tab {
            0 => self.shift_plans_tab(ui),
            1 => self.shift_approval_tab(ui),
            _ => self.shift_report_tab(ui),
        }
    }

    /// Vardiya takvimi: kullanıcılar planı salt-okunur görür; düzenleme yöneticide.
    fn shift_plans_tab(&mut self, ui: &mut egui::Ui) {
        if self.is_admin() {
            egui::CollapsingHeader::new("➕ Vardiya Penceresi Ekle (gün + saat aralığı)")
                .show(ui, |ui| {
                    egui::Grid::new("plan_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                        ui.label("Personel *");
                        let labels: Vec<String> = self
                            .users
                            .iter()
                            .map(|u| if u.full_name.is_empty() { u.username.clone() } else { u.full_name.clone() })
                            .collect();
                        let refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
                        combo_index(ui, "plan_user", &mut self.plan_form.user_idx, &[], &refs);
                        ui.end_row();
                        ui.label("Ay *");
                        ui.add(egui::TextEdit::singleline(&mut self.plan_form.month).desired_width(120.0).hint_text("2026-11"));
                        ui.end_row();
                        ui.label("Haftanın Günleri *");
                        ui.horizontal_wrapped(|ui| {
                            for (i, name) in Self::WEEKDAYS.iter().enumerate() {
                                let day = i + 1;
                                let mut sel = self.plan_form.weekdays.contains(&day);
                                if ui.checkbox(&mut sel, *name).changed() {
                                    if sel {
                                        if !self.plan_form.weekdays.contains(&day) {
                                            self.plan_form.weekdays.push(day);
                                        }
                                    } else {
                                        self.plan_form.weekdays.retain(|d| *d != day);
                                    }
                                }
                            }
                        });
                        ui.end_row();
                        ui.label("Çalışma Saati *");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.plan_form.start_time).desired_width(70.0).hint_text("10:00"));
                            ui.label("—");
                            ui.add(egui::TextEdit::singleline(&mut self.plan_form.end_time).desired_width(70.0).hint_text("14:00"));
                        });
                        ui.end_row();
                        ui.label("Not");
                        ui.add(egui::TextEdit::singleline(&mut self.plan_form.note).desired_width(240.0).hint_text("örn. Açılış vardiyası"));
                        ui.end_row();
                    });
                    if ui.button("Pencere Ekle").clicked() {
                        if self.plan_form.weekdays.is_empty() {
                            self.set_error("En az bir gün seçin".into());
                        } else if let Some(u) = self.users.get(self.plan_form.user_idx) {
                            let weekdays = self
                                .plan_form
                                .weekdays
                                .iter()
                                .map(|d| d.to_string())
                                .collect::<Vec<_>>()
                                .join(",");
                            let input = ShiftPlanInput {
                                user_id: u.id,
                                month: self.plan_form.month.trim().to_string(),
                                work_days: self.plan_form.weekdays.len() as i64,
                                daily_hours: (Self::parse_hhmm(&self.plan_form.end_time)
                                    - Self::parse_hhmm(&self.plan_form.start_time))
                                    .max(0.0),
                                start_time: if self.plan_form.start_time.is_empty() { "09:00".into() } else { self.plan_form.start_time.clone() },
                                end_time: if self.plan_form.end_time.is_empty() { "18:00".into() } else { self.plan_form.end_time.clone() },
                                note: self.plan_form.note.clone(),
                                weekdays,
                            };
                            match self.api.upsert_shift_plan(&input) {
                                Ok(_) => {
                                    self.toast("Vardiya penceresi eklendi", false);
                                    self.plan_form = ShiftPlanForm::default();
                                    self.load_plans();
                                }
                                Err(e) => self.set_error(e),
                            }
                        } else {
                            self.set_error("Önce kullanıcı seçin".into());
                        }
                    }
                });
            ui.add_space(6.0);
        }

        // Plan listesi (herkes görür, salt-okunur)
        let mut del: Option<i64> = None;
        egui::Grid::new("plans_list").num_columns(6).striped(true).spacing([16.0, 6.0]).show(ui, |ui| {
            for h in ["Personel", "Ay", "Günler", "Saatler", "Not", ""] {
                ui.label(egui::RichText::new(h).strong());
            }
            ui.end_row();
            let plans = self.plans.clone();
            for p in &plans {
                ui.label(&p.user_name);
                ui.label(&p.month);
                ui.label(Self::weekday_names(&p.weekdays));
                ui.label(egui::RichText::new(format!("{} — {}", p.start_time, p.end_time)).strong());
                ui.label(egui::RichText::new(&p.note).small().weak());
                if self.is_admin() && ui.small_button("Sil").clicked() {
                    del = Some(p.id);
                }
                ui.end_row();
            }
        });
        if let Some(id) = del {
            match self.api.delete_shift_plan(id) {
                Ok(_) => self.load_plans(),
                Err(e) => self.set_error(e),
            }
        }
        if self.plans.is_empty() {
            ui.label(egui::RichText::new("Henüz vardiya planı tanımlanmadı.").small().weak());
        }
    }

    /// Yönetici: gerçekleşen mesai kaydı girer, çoklu seçip onaylar.
    fn shift_approval_tab(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("➕ Mesai Kaydı Ekle (Gün + Saat)").show(ui, |ui| {
            egui::Grid::new("att_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Personel *");
                let labels: Vec<String> = self
                    .users
                    .iter()
                    .map(|u| if u.full_name.is_empty() { u.username.clone() } else { u.full_name.clone() })
                    .collect();
                let refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
                combo_index(ui, "att_user", &mut self.att_form.user_idx, &[], &refs);
                ui.end_row();
                ui.label("Tarih *");
                Self::date_field(ui, &mut self.att_form.date, "GG-AA-YYYY");
                ui.end_row();
                ui.label("Çalışılan saat");
                ui.add(egui::DragValue::new(&mut self.att_form.hours).range(0.0..=24.0).speed(0.5).suffix(" sa"));
                ui.end_row();
                ui.label("Not");
                ui.add(egui::TextEdit::singleline(&mut self.att_form.note).desired_width(240.0));
                ui.end_row();
            });
            if ui.button("Kaydı Ekle").clicked() {
                if let Some(u) = self.users.get(self.att_form.user_idx) {
                    match Self::iso_from_display(&self.att_form.date) {
                        Some(iso) => {
                            let input = ShiftAttendanceInput {
                                user_id: u.id,
                                date: iso,
                                hours: self.att_form.hours,
                                note: self.att_form.note.clone(),
                            };
                            match self.api.upsert_attendance(&input) {
                                Ok(_) => {
                                    self.toast("Mesai kaydı eklendi", false);
                                    self.att_form = ShiftAttendanceForm::default();
                                    self.load_attendance();
                                }
                                Err(e) => self.set_error(e),
                            }
                        }
                        None => self.set_error("Geçerli bir tarih giriniz (GG-AA-YYYY)".into()),
                    }
                } else {
                    self.set_error("Önce kullanıcı seçin".into());
                }
            }
        });
        ui.separator();

        // Toplu onay araçları
        ui.horizontal(|ui| {
            if ui.button("✔ Seçilenleri Onayla").clicked() && !self.att_sel.is_empty() {
                let ids = self.att_sel.clone();
                match self.api.bulk_approve_attendance(&ids, true) {
                    Ok(_) => {
                        self.toast(format!("{} kayıt onaylandı", ids.len()), false);
                        self.att_sel.clear();
                        self.load_attendance();
                    }
                    Err(e) => self.set_error(e),
                }
            }
            if ui.button("✖ Seçilenlerin Onayını Kaldır").clicked() && !self.att_sel.is_empty() {
                let ids = self.att_sel.clone();
                match self.api.bulk_approve_attendance(&ids, false) {
                    Ok(_) => {
                        self.toast(format!("{} kaydın onayı kaldırıldı", ids.len()), false);
                        self.att_sel.clear();
                        self.load_attendance();
                    }
                    Err(e) => self.set_error(e),
                }
            }
            if ui.small_button("Tümünü Seç").clicked() {
                self.att_sel = self.attendance.iter().map(|a| a.id).collect();
            }
            if ui.small_button("Seçimi Temizle").clicked() {
                self.att_sel.clear();
            }
        });

        let mut toggle: Option<i64> = None;
        egui::Grid::new("att_list").num_columns(6).striped(true).spacing([16.0, 6.0]).show(ui, |ui| {
            for h in ["Seç", "Personel", "Tarih", "Saat", "Onay", "Not"] {
                ui.label(egui::RichText::new(h).strong());
            }
            ui.end_row();
            let items = self.attendance.clone();
            for a in &items {
                let mut sel = self.att_sel.contains(&a.id);
                if ui.checkbox(&mut sel, "").changed() {
                    if sel {
                        if !self.att_sel.contains(&a.id) {
                            self.att_sel.push(a.id);
                        }
                    } else {
                        self.att_sel.retain(|id| *id != a.id);
                    }
                }
                ui.label(&a.user_name);
                // Tarihi GG-AA-YYYY göster
                let disp = format!("{}-{}-{}", &a.date[8..10], &a.date[5..7], &a.date[0..4]);
                ui.label(disp);
                ui.label(format!("{:.1} sa", a.hours));
                let mut approved = a.approved;
                if ui.checkbox(&mut approved, "").changed() {
                    toggle = Some(a.id);
                }
                ui.label(egui::RichText::new(&a.note).small().weak());
                ui.end_row();
            }
        });
        if let Some(id) = toggle {
            match self.api.toggle_attendance(id) {
                Ok(_) => self.load_attendance(),
                Err(e) => self.set_error(e),
            }
        }
        if self.attendance.is_empty() {
            ui.label(egui::RichText::new("Henüz mesai kaydı yok.").small().weak());
        }
    }

    /// Haftalık/aylık vardiya-mesai raporu (yönetici).
    fn shift_report_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Haftalık / Aylık Vardiya Raporu").strong());
        ui.label(egui::RichText::new("Ay seçin, personeli işaretleyin; planlanan-gerçekleşen çalışma ve devam oranını PDF/CSV olarak alın.").small().weak());
        ui.separator();

        egui::Grid::new("rep_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label("Ay *");
            ui.add(egui::TextEdit::singleline(&mut self.report_month).desired_width(120.0).hint_text("2026-11"));
            ui.end_row();
            ui.label("Personel *");
            ui.horizontal_wrapped(|ui| {
                for u in self.users.clone() {
                    let name = if u.full_name.is_empty() { u.username.clone() } else { u.full_name.clone() };
                    let mut sel = self.report_sel.contains(&u.id);
                    if ui.checkbox(&mut sel, name).changed() {
                        if sel {
                            if !self.report_sel.contains(&u.id) {
                                self.report_sel.push(u.id);
                            }
                        } else {
                            self.report_sel.retain(|id| *id != u.id);
                        }
                    }
                }
            });
            ui.end_row();
        });
        ui.horizontal(|ui| {
            if ui.small_button("Tümünü Seç").clicked() {
                self.report_sel = self.users.iter().map(|u| u.id).collect();
            }
            if ui.small_button("Seçimi Temizle").clicked() {
                self.report_sel.clear();
            }
        });

        let rows = self.compute_report_rows();
        if !rows.is_empty() {
            ui.add_space(8.0);
            egui::Grid::new("rep_table").num_columns(7).striped(true).spacing([14.0, 5.0]).show(ui, |ui| {
                for h in ["Personel", "Plan. Gün", "Plan. Saat", "Gerç. Gün", "Toplam Saat", "Onaylı", "Devam %"] {
                    ui.label(egui::RichText::new(h).strong());
                }
                ui.end_row();
                for r in &rows {
                    ui.label(&r.name);
                    ui.label(r.planned_days.to_string());
                    ui.label(format!("{:.1}", r.planned_hours));
                    ui.label(r.actual_days.to_string());
                    ui.label(format!("{:.1}", r.actual_hours));
                    ui.label(r.approved_days.to_string());
                    ui.label(egui::RichText::new(format!("{:.0}%", r.rate())).color(
                        if r.rate() >= 80.0 { egui::Color32::from_rgb(60, 160, 90) } else { egui::Color32::from_rgb(220, 140, 40) },
                    ));
                    ui.end_row();
                }
            });
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("📄 PDF Rapor Oluştur").clicked() {
                self.export_report(&rows, "pdf");
            }
            if ui.button("📊 CSV Rapor Oluştur (Excel)").clicked() {
                self.export_report(&rows, "csv");
            }
        });
    }

    fn compute_report_rows(&self) -> Vec<crate::export::ReportRow> {
        let month = self.report_month.trim().to_string();
        let days = Self::month_days(&month);
        let mut rows = Vec::new();
        for uid in &self.report_sel {
            let user = match self.users.iter().find(|u| u.id == *uid) {
                Some(u) => u,
                None => continue,
            };
            let mut planned_days = 0i64;
            let mut planned_hours = 0.0f64;
            for p in self.plans.iter().filter(|p| p.user_id == *uid && p.month == month) {
                let selected: Vec<usize> = p
                    .weekdays
                    .split(',')
                    .filter_map(|x| x.trim().parse::<usize>().ok())
                    .collect();
                let dur = (Self::parse_hhmm(&p.end_time) - Self::parse_hhmm(&p.start_time)).max(0.0);
                for d in &days {
                    let wd = d.weekday().number_from_monday() as usize;
                    if selected.contains(&wd) {
                        planned_days += 1;
                        planned_hours += dur;
                    }
                }
            }
            let mut actual_days = 0i64;
            let mut actual_hours = 0.0f64;
            let mut approved_days = 0i64;
            for a in self.attendance.iter().filter(|a| a.user_id == *uid && a.date.starts_with(&month)) {
                actual_days += 1;
                actual_hours += a.hours;
                if a.approved {
                    approved_days += 1;
                }
            }
            rows.push(crate::export::ReportRow {
                name: if user.full_name.is_empty() { user.username.clone() } else { user.full_name.clone() },
                planned_days,
                planned_hours,
                actual_days,
                actual_hours,
                approved_days,
            });
        }
        rows.sort_by(|a, b| a.name.cmp(&b.name));
        rows
    }

    fn export_report(&mut self, rows: &[crate::export::ReportRow], kind: &str) {
        if rows.is_empty() {
            self.set_error("Rapor için önce ay girip personel seçin".into());
            return;
        }
        let dir = export_dir();
        let _ = std::fs::create_dir_all(&dir);
        let month = self.report_month.trim().to_string();
        let safe_month = month.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect::<String>();
        let result = if kind == "pdf" {
            crate::export::export_shift_report_pdf(&dir.join(format!("vardiya-rapor-{safe_month}.pdf")), &month, rows)
        } else {
            crate::export::export_shift_report_csv(&dir.join(format!("vardiya-rapor-{safe_month}.csv")), &month, rows)
        };
        match result {
            Ok(path) => {
                self.toast(format!("Rapor oluşturuldu: {}", path.display()), false);
                let _ = open::that(&path);
            }
            Err(e) => self.set_error(format!("Rapor oluşturulamadı: {e}")),
        }
    }

    // =====================================================================
    //  CRM / KULLANICILAR / AYARLAR / PROFİL
    // =====================================================================
    fn crm_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if !self.is_admin() {
            ui.label("Bu bölüm yalnızca yöneticiye açıktır.");
            return;
        }
        ui.horizontal(|ui| {
            ui.heading("CRM Kayıtları (Taşınmaz Sahipleri / Müşteriler)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let prevd = self.show_deleted;
                ui.checkbox(&mut self.show_deleted, "🗑 Silinmişleri göster");
                if prevd != self.show_deleted {
                    self.load_crm();
                }
                ui.separator();
                ui.selectable_value(&mut self.crm_list_view, false, "▦ Görsel");
                ui.selectable_value(&mut self.crm_list_view, true, "☰ Liste");
            });
        });
        ui.separator();
        let mut refresh = false;

        // Kayıt / düzenleme formu
        let editing = self.crm_form.edit_id.is_some();
        let title = if editing { format!("✏ Düzenle: #{}", self.crm_form.edit_id.unwrap()) } else { "➕ Yeni CRM Kaydı".to_string() };
        egui::CollapsingHeader::new(title).default_open(editing).show(ui, |ui| {
            ui.horizontal(|ui| {
                // Profil fotoğrafı (düzenleme modunda)
                ui.vertical(|ui| {
                    if let Some(photo) = self.crm_photo_texture(ctx) {
                        ui.add(
                            egui::Image::new(egui::load::SizedTexture::new(photo.id(), egui::vec2(84.0, 84.0)))
                                .rounding(egui::Rounding::same(42.0)),
                        );
                    } else {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(84.0, 84.0), egui::Sense::hover());
                        ui.painter_at(rect).rect_filled(rect, 42.0, egui::Color32::from_gray(70));
                    }
                    if editing && ui.small_button("Fotoğraf").clicked() {
                        if let Some(files) = pick_files() {
                            if let Some(f) = files.into_iter().next() {
                                let id = self.crm_form.edit_id.unwrap();
                                match self.api.upload_crm_photo(id, &f) {
                                    Ok(rec) => {
                                        if let Some(p) = &rec.photo {
                                            self.textures.remove(p);
                                        }
                                        self.toast("Profil fotoğrafı güncellendi (256×256 kırpıldı)", false);
                                        refresh = true;
                                    }
                                    Err(e) => self.set_error(e),
                                }
                            }
                        }
                    }
                });
                ui.add_space(8.0);
                egui::Grid::new("crm_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label("Ad Soyad *");
                    let before = self.crm_form.full_name.clone();
                    ui.add(egui::TextEdit::singleline(&mut self.crm_form.full_name).desired_width(300.0));
                    if self.crm_form.full_name != before {
                        self.crm_form.full_name = tr_title_case(&self.crm_form.full_name);
                    }
                    ui.end_row();
                    ui.label("Telefon");
                    ui.add(egui::TextEdit::singleline(&mut self.crm_form.phone).desired_width(300.0));
                    ui.end_row();
                    ui.label("E-posta");
                    ui.add(egui::TextEdit::singleline(&mut self.crm_form.email).desired_width(300.0));
                    ui.end_row();
                    ui.label("Tip");
                    ui.add(egui::TextEdit::singleline(&mut self.crm_form.owner_type).desired_width(300.0).hint_text("Sahip / Yetkili / Müşteri"));
                    ui.end_row();
                    ui.label("Adres");
                    ui.add(egui::TextEdit::singleline(&mut self.crm_form.address).desired_width(300.0));
                    ui.end_row();
                    ui.label("Notlar");
                    ui.add(egui::TextEdit::multiline(&mut self.crm_form.notes).desired_width(300.0).desired_rows(2));
                    ui.end_row();
                });
            });
            ui.horizontal(|ui| {
                let label = if editing { "Değişiklikleri Kaydet" } else { "Kaydet" };
                if ui.button(label).clicked() {
                    if self.crm_form.full_name.trim().is_empty() {
                        self.set_error("Ad Soyad zorunludur".into());
                    } else {
                        let input = CrmInput {
                            full_name: self.crm_form.full_name.clone(),
                            phone: self.crm_form.phone.clone(),
                            email: self.crm_form.email.clone(),
                            address: self.crm_form.address.clone(),
                            owner_type: if self.crm_form.owner_type.is_empty() { "Sahip".into() } else { self.crm_form.owner_type.clone() },
                            notes: self.crm_form.notes.clone(),
                        };
                        let result = match self.crm_form.edit_id {
                            Some(id) => self.api.update_crm(id, &input).map(|_| ()),
                            None => self.api.create_crm(&input).map(|_| ()),
                        };
                        match result {
                            Ok(_) => {
                                self.toast(if editing { "CRM kaydı güncellendi" } else { "CRM kaydı eklendi" }, false);
                                self.crm_form = CrmForm::default();
                                refresh = true;
                            }
                            Err(e) => self.set_error(e),
                        }
                    }
                }
                if editing && ui.button("İptal").clicked() {
                    self.crm_form = CrmForm::default();
                }
            });
        });
        ui.separator();

        let mut act: Option<(i64, bool)> = None; // (id, is_delete)
        if self.crm_list_view {
            // Görsel (thumbnail) görünüm — varsayılan
            let card_w = 210.0;
            let cols = ((ui.available_width() / (card_w + 10.0)).floor() as usize).max(1);
            let mut idx = 0;
            let crm_items = self.crm.clone();
            while idx < crm_items.len() {
                ui.horizontal(|ui| {
                    for _ in 0..cols {
                        if idx >= crm_items.len() { break; }
                        let c = crm_items[idx].clone();
                        egui::Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_width(card_w);
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    if let Some(tex) = self.crm_card_texture(ctx, &c) {
                                        ui.add(
                                            egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(56.0, 56.0)))
                                                .rounding(egui::Rounding::same(28.0)),
                                        );
                                    } else {
                                        let (rect, _) = ui.allocate_exact_size(egui::vec2(56.0, 56.0), egui::Sense::hover());
                                        ui.painter_at(rect).rect_filled(rect, 28.0, egui::Color32::from_gray(70));
                                    }
                                    ui.vertical(|ui| {
                                        ui.add(egui::Label::new(egui::RichText::new(&c.full_name).strong()).truncate());
                                        ui.add(egui::Label::new(egui::RichText::new(&c.owner_type).small().weak()).truncate());
                                        ui.add(egui::Label::new(egui::RichText::new(&c.phone).small()).truncate());
                                    });
                                });
                                ui.add(egui::Label::new(egui::RichText::new(&c.email).small().weak()).truncate());
                                if c.is_deleted {
                                    ui.label(egui::RichText::new("🗑 silinmiş").small().color(egui::Color32::from_rgb(220, 80, 80)));
                                }
                                ui.horizontal(|ui| {
                                    if ui.small_button("Düzenle").clicked() {
                                        self.crm_form = CrmForm {
                                            edit_id: Some(c.id),
                                            full_name: c.full_name.clone(),
                                            phone: c.phone.clone(),
                                            email: c.email.clone(),
                                            address: c.address.clone(),
                                            owner_type: c.owner_type.clone(),
                                            notes: c.notes.clone(),
                                        };
                                    }
                                    if !c.is_deleted && ui.small_button("Sil").clicked() {
                                        act = Some((c.id, true));
                                    }
                                    if c.is_deleted && ui.small_button("♻ Geri Yükle").clicked() {
                                        act = Some((c.id, false));
                                    }
                                });
                            });
                        });
                        idx += 1;
                    }
                });
                ui.add_space(4.0);
            }
        } else {
            // Liste görünümü
            egui::Grid::new("crm_list").num_columns(7).striped(true).spacing([14.0, 6.0]).show(ui, |ui| {
                for h in ["", "Ad Soyad", "Tip", "Telefon", "E-posta", "Ekleyen / Tarih", "İşlem"] {
                    ui.label(egui::RichText::new(h).strong());
                }
                ui.end_row();
                let crm_items = self.crm.clone();
                for c in &crm_items {
                    if let Some(tex) = self.crm_card_texture(ui.ctx(), c) {
                        ui.add(
                            egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(34.0, 34.0)))
                                .rounding(egui::Rounding::same(17.0)),
                        );
                    } else {
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(34.0, 34.0), egui::Sense::hover());
                        ui.painter_at(rect).rect_filled(rect, 17.0, egui::Color32::from_gray(70));
                    }
                    let name = if c.is_deleted {
                        egui::RichText::new(format!("{} 🗑", c.full_name)).strikethrough()
                    } else {
                        egui::RichText::new(&c.full_name)
                    };
                    ui.label(name);
                    ui.label(&c.owner_type);
                    ui.label(&c.phone);
                    ui.label(&c.email);
                    ui.label(egui::RichText::new(format!("{}\n{}", c.created_by_name, c.created_at)).small());
                    ui.horizontal(|ui| {
                        if ui.small_button("Düzenle").clicked() {
                            self.crm_form = CrmForm {
                                edit_id: Some(c.id),
                                full_name: c.full_name.clone(),
                                phone: c.phone.clone(),
                                email: c.email.clone(),
                                address: c.address.clone(),
                                owner_type: c.owner_type.clone(),
                                notes: c.notes.clone(),
                            };
                        }
                        if !c.is_deleted && ui.small_button("Sil").clicked() {
                            act = Some((c.id, true));
                        }
                        if c.is_deleted && ui.small_button("♻").clicked() {
                            act = Some((c.id, false));
                        }
                    });
                    ui.end_row();
                }
            });
        }
        if let Some((id, is_delete)) = act {
            let result = if is_delete { self.api.delete_crm(id) } else { self.api.restore_crm(id) };
            match result {
                Ok(_) => {
                    self.toast(if is_delete { "CRM kaydı silme komutu aldı (yönetici bilgilendirildi)" } else { "CRM kaydı geri yüklendi" }, false);
                    refresh = true;
                }
                Err(e) => self.set_error(e),
            }
        }
        if refresh {
            self.load_crm();
        }
    }

    /// Düzenleme formundaki CRM fotoğrafının dokusu.
    fn crm_photo_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        let photo = self.crm_form.edit_id
            .and_then(|id| self.crm.iter().find(|c| c.id == id))
            .and_then(|c| c.photo.clone())?;
        self.texture(ctx, &photo)
    }

    /// CRM kartındaki fotoğraf dokusu.
    fn crm_card_texture(&mut self, ctx: &egui::Context, c: &CrmRecord) -> Option<egui::TextureHandle> {
        let photo = c.photo.clone()?;
        self.texture(ctx, &photo)
    }

    fn users_screen(&mut self, ui: &mut egui::Ui) {
        if !self.is_admin() {
            ui.label("Bu bölüm yalnızca yöneticiye açıktır.");
            return;
        }
        ui.heading("Kullanıcılar (Çalışanlar)");
        ui.separator();
        self.users_manager_ui(ui);
    }

    /// Kullanıcı yönetimi (ayarlar panelindeki sekmeden de kullanılır).
    fn users_manager_ui(&mut self, ui: &mut egui::Ui) {
        let mut refresh = false;

        egui::CollapsingHeader::new("➕ Yeni Kullanıcı").show(ui, |ui| {
            egui::Grid::new("user_grid").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label("Kullanıcı Adı *");
                ui.add(egui::TextEdit::singleline(&mut self.user_form.username).desired_width(300.0));
                ui.end_row();
                ui.label("Parola *");
                ui.add(egui::TextEdit::singleline(&mut self.user_form.password).desired_width(300.0).password(true));
                ui.end_row();
                ui.label("Ad Soyad");
                ui.add(egui::TextEdit::singleline(&mut self.user_form.full_name).desired_width(300.0));
                ui.end_row();
                ui.label("Telefon");
                ui.add(egui::TextEdit::singleline(&mut self.user_form.phone).desired_width(300.0));
                ui.end_row();
                ui.label("E-posta");
                ui.add(egui::TextEdit::singleline(&mut self.user_form.email).desired_width(300.0));
                ui.end_row();
                ui.label("Rol");
                ui.checkbox(&mut self.user_form.role_is_admin, "Yönetici (admin)");
                ui.end_row();
            });
            if ui.button("Kullanıcı Oluştur").clicked() {
                let input = CreateUserRequest {
                    username: self.user_form.username.clone(),
                    password: self.user_form.password.clone(),
                    full_name: self.user_form.full_name.clone(),
                    role: if self.user_form.role_is_admin { Role::Admin } else { Role::Agent },
                    phone: self.user_form.phone.clone(),
                    email: self.user_form.email.clone(),
                };
                match self.api.create_user(&input) {
                    Ok(_) => { self.toast("Kullanıcı oluşturuldu", false); self.user_form = UserForm::default(); refresh = true; }
                    Err(e) => self.set_error(e),
                }
            }
        });
        ui.separator();

        let mut toggle: Option<i64> = None;
        egui::Grid::new("users_list").num_columns(6).striped(true).spacing([14.0, 6.0]).show(ui, |ui| {
            for h in ["Ad Soyad", "Kullanıcı", "Rol", "Telefon", "Durum", ""] {
                ui.label(egui::RichText::new(h).strong());
            }
            ui.end_row();
            for u in &self.users {
                ui.label(if u.full_name.is_empty() { &u.username } else { &u.full_name });
                ui.label(&u.username);
                ui.label(u.role.label());
                ui.label(&u.phone);
                ui.label(if u.active { "Aktif" } else { "Pasif" });
                if ui.small_button(if u.active { "Pasifleştir" } else { "Aktifleştir" }).clicked() {
                    toggle = Some(u.id);
                }
                ui.end_row();
            }
        });
        if let Some(id) = toggle {
            let _ = self.api.toggle_user(id);
            refresh = true;
        }
        if refresh {
            self.load_users();
        }
    }

    fn settings_screen(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ayarlar Paneli");
        ui.separator();

        let tabs: [(SettingsTab, &str); 6] = [
            (SettingsTab::View, "🎨 Görünüm"),
            (SettingsTab::Output, "📄 Çıktı"),
            (SettingsTab::Validation, "✅ Doğrulama"),
            (SettingsTab::Users, "🧑‍💼 Kullanıcılar"),
            (SettingsTab::Help, "❓ Yardım İçerikleri"),
            (SettingsTab::Test, "🧪 Sistem Testi"),
        ];

        ui.horizontal(|ui| {
            for (tab, label) in tabs {
                let admin_only = !matches!(tab, SettingsTab::View);
                if admin_only && !self.is_admin() {
                    continue;
                }
                if ui
                    .selectable_label(self.settings_tab == tab, label)
                    .clicked()
                {
                    self.settings_tab = tab;
                }
            }
        });
        ui.separator();

        match self.settings_tab {
            SettingsTab::View => self.settings_view_tab(ui),
            SettingsTab::Output => self.settings_output_tab(ui),
            SettingsTab::Validation => self.settings_validation_tab(ui),
            SettingsTab::Users => self.settings_users_tab(ui),
            SettingsTab::Help => self.settings_help_tab(ui),
            SettingsTab::Test => self.settings_test_tab(ui),
        }
    }

    /// Yardım içeriklerini görüntüleme/düzenleme (yönetici).
    fn settings_help_tab(&mut self, ui: &mut egui::Ui) {
        if !self.is_admin() {
            ui.label("Bu sekme yalnızca yöneticiye açıktır.");
            return;
        }
        ui.label(egui::RichText::new("Ekran Yardım İçerikleri").strong());
        ui.label(
            egui::RichText::new("Her ekranın sağ üstündeki ? düğmesinde görünen açıklamalar. Düzenleyip kaydedin; değişiklik anında tüm kullanıcılara yansır.")
                .small()
                .weak(),
        );
        ui.separator();
        if self.help_topics.is_empty() {
            if ui.button("İçerikleri Yükle").clicked() {
                self.load_help();
            }
            return;
        }
        let topics = self.help_topics.clone();
        for t in topics {
            egui::CollapsingHeader::new(format!("📄 {}", t.title)).show(ui, |ui| {
                let mut title = t.title.clone();
                let mut body = t.body.clone();
                egui::Grid::new(format!("help_g_{}", t.key)).num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                    ui.label("Başlık");
                    ui.add(egui::TextEdit::singleline(&mut title).desired_width(420.0));
                    ui.end_row();
                    ui.label("İçerik");
                    ui.add(
                        egui::TextEdit::multiline(&mut body)
                            .desired_width(420.0)
                            .desired_rows(5)
                            .font(egui::TextStyle::Monospace),
                    );
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    if ui.button("💾 Kaydet").clicked() {
                        let topic = HelpTopic { key: t.key.clone(), title, body };
                        match self.api.update_help(&topic) {
                            Ok(_) => {
                                self.toast("Yardım içeriği güncellendi", false);
                                self.load_help();
                            }
                            Err(e) => self.set_error(e),
                        }
                    }
                    ui.label(egui::RichText::new(format!("anahtar: {}", t.key)).small().weak());
                });
            });
        }
    }

    /// Tek tuşla tüm modül testleri + sağlık/ajanda logu (yönetici).
    fn settings_test_tab(&mut self, ui: &mut egui::Ui) {
        if !self.is_admin() {
            ui.label("Bu sekme yalnızca yöneticiye açıktır.");
            return;
        }
        ui.label(egui::RichText::new("Sistem Testi ve Sağlık Kontrolü").strong());
        ui.label(
            egui::RichText::new("Sunucu bağlantısı, oturum, tüm modüller ve yerel ortam tek tuşla test edilir. Logu kopyalayıp yapay zekâya yapıştırarak çözüm adımları isteyebilirsiniz.")
                .small()
                .weak(),
        );
        ui.separator();
        ui.horizontal(|ui| {
            if ui.add(egui::Button::new(egui::RichText::new("▶ Testi Çalıştır").strong())).clicked() {
                self.run_system_tests();
            }
            if ui.button("📋 Kopyala").clicked() {
                ui.ctx().copy_text(self.test_log.clone());
                self.toast("Log panoya kopyalandı", false);
            }
            if ui.button("🗑 Temizle").clicked() {
                self.test_log.clear();
            }
        });
        ui.add_space(6.0);
        egui::ScrollArea::vertical().max_height(ui.available_height() - 10.0).show(ui, |ui| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                if self.test_log.is_empty() {
                    ui.label(egui::RichText::new("Henüz test çalıştırılmadı. '▶ Testi Çalıştır' düğmesine basın.").weak());
                } else {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.test_log)
                            .desired_width(ui.available_width() - 16.0)
                            .desired_rows(24)
                            .font(egui::TextStyle::Monospace),
                    );
                }
            });
        });
    }

    fn run_system_tests(&mut self) {
        use std::fmt::Write as _;
        let mut log = String::new();
        let _ = writeln!(log, "==== Gayrimenkul360 Sistem Testi ====");
        let _ = writeln!(log, "Tarih: {}  |  Sürüm: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), env!("CARGO_PKG_VERSION"));
        let _ = writeln!(log, "Sunucu: {}", self.api.base);
        let mut passed = 0usize;
        let mut total = 0usize;

        let mut step = |log: &mut String, name: &str, ok: bool, detail: String, passed: &mut usize, total: &mut usize| {
            *total += 1;
            if ok {
                *passed += 1;
                let _ = writeln!(log, "[OK  ] {}  {}", name, detail);
            } else {
                let _ = writeln!(log, "[HATA] {}  {}", name, detail);
            }
        };

        // 1) Sunucu bağlantısı + gecikme
        let t0 = std::time::Instant::now();
        let ok = self.api.health();
        let ms = t0.elapsed().as_millis();
        step(&mut log, "SUNUCU", ok, format!("bağlantı {} ms", ms), &mut passed, &mut total);

        // 2) Oturum
        match self.api.me() {
            Ok(u) => { step(&mut log, "OTURUM", true, format!("{} ({})", u.username, u.role.label()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "OTURUM", false, e, &mut passed, &mut total); }
        }

        // 3) Kullanıcılar
        match self.api.list_users() {
            Ok(us) => { step(&mut log, "KULLANICI", true, format!("{} kayıt", us.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "KULLANICI", false, e, &mut passed, &mut total); }
        }

        // 4) Portföy listesi
        match self.api.list_portfolios(None, None, None, None, None, None, None, false) {
            Ok(ps) => { step(&mut log, "PORTFÖY", true, format!("{} kayıt", ps.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "PORTFÖY", false, e, &mut passed, &mut total); }
        }

        // 5) Portföy yazma testi (oluştur → doğrula → mantıksal sil)
        let ts = chrono::Local::now().format("%H%M%S").to_string();
        let test_title = format!("[TEST] Sistem Kontrolü {ts}");
        let input = PortfolioInput {
            title: test_title.clone(),
            listing_type: "satilik".into(),
            category: "konut".into(),
            status: "taslak".into(),
            price: 1.0,
            currency: "TRY".into(),
            area_m2: 1.0,
            area_net: 1.0,
            ada_parsel: String::new(),
            room_count: String::new(),
            building_age: String::new(),
            floor: String::new(),
            total_floors: String::new(),
            heating: String::new(),
            furnished: false,
            il: "Muğla".into(),
            ilce: "Fethiye".into(),
            mahalle: String::new(),
            address: String::new(),
            latitude: 0.0,
            longitude: 0.0,
            description: "Sistem testi tarafından oluşturuldu".into(),
            owner_crm_id: None,
        };
        match self.api.create_portfolio(&input) {
            Ok(p) => {
                let verified = self.api.get_portfolio(p.id).map(|g| g.title == test_title).unwrap_or(false);
                step(&mut log, "PORTFÖY-YAZ", verified, format!("id={} oluşturuldu ve doğrulandı", p.id), &mut passed, &mut total);
                let _ = self.api.delete_portfolio(p.id);
                let _ = writeln!(log, "[BİLGİ] Test portföyü mantıksal silindi; 'Silinmişleri göster' ile kalıcı temizleyebilirsiniz.");
            }
            Err(e) => { step(&mut log, "PORTFÖY-YAZ", false, e, &mut passed, &mut total); }
        }

        // 6) CRM
        match self.api.list_crm(false) {
            Ok(c) => { step(&mut log, "CRM", true, format!("{} kayıt", c.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "CRM", false, e, &mut passed, &mut total); }
        }

        // 7) Görevler
        match self.api.list_tasks() {
            Ok(t) => { step(&mut log, "GÖREV", true, format!("{} kayıt", t.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "GÖREV", false, e, &mut passed, &mut total); }
        }

        // 8) Hedefler
        match self.api.list_goals() {
            Ok(g) => { step(&mut log, "HEDEF", true, format!("{} kayıt", g.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "HEDEF", false, e, &mut passed, &mut total); }
        }

        // 9) Vardiya
        match self.api.list_shift_plans() {
            Ok(p) => { step(&mut log, "VARDİYA", true, format!("{} plan satırı", p.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "VARDİYA", false, e, &mut passed, &mut total); }
        }
        match self.api.list_attendance() {
            Ok(a) => { step(&mut log, "MESAİ-KAYIT", true, format!("{} kayıt", a.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "MESAİ-KAYIT", false, e, &mut passed, &mut total); }
        }

        // 10) Bildirimler
        match self.api.list_notifications() {
            Ok(n) => { step(&mut log, "BİLDİRİM", true, format!("{} kayıt", n.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "BİLDİRİM", false, e, &mut passed, &mut total); }
        }

        // 11) Ayarlar
        match self.api.get_settings() {
            Ok(_) => { step(&mut log, "AYARLAR", true, "GET ok".into(), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "AYARLAR", false, e, &mut passed, &mut total); }
        }

        // 12) Yardım içerikleri
        match self.api.list_help() {
            Ok(h) => { step(&mut log, "YARDIM", true, format!("{} konu", h.len()), &mut passed, &mut total); }
            Err(e) => { step(&mut log, "YARDIM", false, e, &mut passed, &mut total); }
        }

        // 13) Yerel ortam: yapılandırma yazılabilir mi, çıktı klasörü oluşabilir mi
        let cfg_ok = std::fs::write(config_file(), "{}").is_ok();
        step(&mut log, "YEREL-CONFIG", cfg_ok, config_file().display().to_string(), &mut passed, &mut total);
        let dir_ok = std::fs::create_dir_all(export_dir()).is_ok();
        step(&mut log, "YEREL-ÇIKTI", dir_ok, export_dir().display().to_string(), &mut passed, &mut total);

        let _ = writeln!(log, "------------------------------------");
        let _ = writeln!(log, "ÖZET: {passed}/{total} test geçti");
        if passed != total {
            let _ = writeln!(log, "Hata çözümü için bu logun tamamını kopyalayıp yapay zekâ asistanına yapıştırabilirsiniz.");
        }
        self.test_log = log;
        self.test_running = false;
    }

    /// Görünüm sekmesi: tema ve ölçeklendirme (tüm kullanıcılara açık, yerel kayıtlı).
    fn settings_view_tab(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.label(egui::RichText::new("Görünüm (bu bilgisayara özgü)").strong());
        ui.label(
            egui::RichText::new(
                "Tema ve yakınlaştırma tercihi bu bilgisayarda saklanır; diğer kullanıcıları etkilemez.",
            )
            .small()
            .weak(),
        );
        ui.add_space(8.0);

        ui.label("Tema:");
        ui.horizontal(|ui| {
            for (val, label) in [
                ("dark", "🌙 Karanlık Tema"),
                ("light", "☀️ Aydınlık Tema"),
                ("system", "🖥 Sistem Tercihine Uy"),
            ] {
                if ui.selectable_label(self.theme == val, label).clicked() {
                    self.theme = val.to_string();
                    self.local_theme_set = true;
                    self.apply_theme(&ctx);
                    self.apply_accent(&ctx);
                    self.save_config();
                }
            }
        });

        ui.add_space(14.0);
        ui.label("Yazı Tipi:");
        egui::Grid::new("font_grid").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
            ui.label("Yazı Tipi Ailesi");
            combo_index(
                ui,
                "font_family",
                &mut self.font_family,
                &[],
                &["Roboto (önerilen)", "Sistem Fontu"],
            );
            ui.end_row();
            ui.label("Yazı Boyutu (punto)");
            let mut fs = self.font_size;
            let resp = ui.add(
                egui::Slider::new(&mut fs, 10.0..=24.0)
                    .step_by(0.5)
                    .text(format!("{:.1} pt", self.font_size)),
            );
            if resp.changed() {
                self.font_size = fs;
                self.apply_font_size(&ctx);
                self.save_config();
            }
            ui.end_row();
        });
        if ui.selectable_label(self.font_family == 1, "Sistem fontuna geç").clicked() {
            self.font_family = 1;
            self.apply_fonts(&ctx);
            self.apply_font_size(&ctx);
            self.save_config();
        } else if ui.selectable_label(self.font_family == 0, "Roboto'ya dön").clicked() {
            self.font_family = 0;
            self.apply_fonts(&ctx);
            self.apply_font_size(&ctx);
            self.save_config();
        }

        ui.add_space(14.0);
        ui.label("Renk Teması:");
        ui.horizontal_wrapped(|ui| {
            let presets: [(&str, &str); 5] = [
                ("blue", "🔵 Mavi"),
                ("green", "🟢 Yeşil"),
                ("gray", "⚪ Gri"),
                ("orange", "🟠 Turuncu"),
                ("purple", "🟣 Mor"),
            ];
            for (val, label) in presets {
                if ui.selectable_label(self.accent == val, label).clicked() {
                    self.accent = val.to_string();
                    self.local_theme_set = true;
                    self.apply_accent(&ctx);
                    self.save_config();
                }
            }
            if ui.button("⚪ Gri (Açık Tema)").clicked() {
                self.accent = "gray".to_string();
                self.theme = "light".to_string();
                self.local_theme_set = true;
                self.apply_theme(&ctx);
                self.apply_accent(&ctx);
                self.save_config();
            }
        });

        ui.add_space(14.0);
        ui.label("Yakınlaştırma / UI Ölçeği (tüm bileşenleri büyütür-küçültür):");
        let mut z = self.zoom;
        let resp = ui.add(
            egui::Slider::new(&mut z, 0.6..=2.5)
                .step_by(0.05)
                .text(format!("{:.0}%", self.zoom * 100.0)),
        );
        if resp.changed() {
            self.set_zoom(&ctx, z);
        }
        ui.horizontal(|ui| {
            if ui.small_button("− Küçült").clicked() {
                self.set_zoom(&ctx, self.zoom - 0.1);
            }
            if ui.small_button("+ Büyüt").clicked() {
                self.set_zoom(&ctx, self.zoom + 0.1);
            }
            if ui.small_button("⟲ %100 Sıfırla").clicked() {
                self.set_zoom(&ctx, 1.0);
            }
        });
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new("Kısayollar: Ctrl + (büyüt) • Ctrl − (küçült) • Ctrl 0 (sıfırla)")
                .small()
                .weak(),
        );
    }

    /// Çıktı sekmesi (yalnızca yönetici).
    fn settings_output_tab(&mut self, ui: &mut egui::Ui) {
        if !self.is_admin() {
            ui.label("Bu sekme yalnızca yöneticiye açıktır.");
            return;
        }
        ui.label(egui::RichText::new("Çıktı Ayarları").strong());
        ui.label(
            egui::RichText::new("PDF/DOCX çıktılarında ve paylaşımlarda hangi bilgilerin görüneceğini belirleyin.")
                .small()
                .weak(),
        );
        ui.separator();
        let s = &mut self.settings;
        ui.checkbox(&mut s.show_name, "İsim / Yetkili bilgisi");
        ui.checkbox(&mut s.show_phone, "Telefon bilgisi");
        ui.checkbox(&mut s.show_email, "E-posta bilgisi");
        ui.checkbox(&mut s.show_price, "Fiyat bilgisi");
        ui.checkbox(&mut s.show_address, "Adres bilgisi");
        ui.checkbox(&mut s.show_description, "Açıklama");
        ui.checkbox(&mut s.show_cover_image, "Kapak resmi (DOCX)");
        ui.add_space(8.0);
        ui.label("Başlık metni:");
        ui.add(egui::TextEdit::singleline(&mut s.header_text).desired_width(360.0));
        ui.label("Alt bilgi metni:");
        ui.add(egui::TextEdit::multiline(&mut s.footer_text).desired_width(360.0).desired_rows(2));

        ui.add_space(14.0);
        ui.label(egui::RichText::new("Yeni Kullanıcılar İçin Varsayılan Görünüm").strong());
        ui.label(
            egui::RichText::new("Bu bilgisayarda yerel tercih kaydetmemiş kullanıcılar bu ayarlarla başlar.")
                .small()
                .weak(),
        );
        egui::Grid::new("ui_defaults_grid").num_columns(2).spacing([16.0, 6.0]).show(ui, |ui| {
            ui.label("Varsayılan tema");
            let mut theme_idx = match self.settings.default_theme.as_str() {
                "dark" => 1,
                "light" => 2,
                _ => 0,
            };
            combo_index(ui, "def_theme", &mut theme_idx, &[], &["Sistem tercihi", "Karanlık", "Aydınlık"]);
            self.settings.default_theme = match theme_idx {
                1 => "dark".to_string(),
                2 => "light".to_string(),
                _ => "system".to_string(),
            };
            ui.end_row();
            ui.label("Varsayılan ölçek");
            let zoom_pct = format!("{:.0}%", self.settings.default_zoom * 100.0);
            ui.add(
                egui::Slider::new(&mut self.settings.default_zoom, 0.6..=2.5)
                    .step_by(0.05)
                    .text(zoom_pct),
            );
            ui.end_row();
        });

        ui.add_space(12.0);
        if ui.button("Ayarları Kaydet").clicked() {
            let settings = self.settings.clone();
            match self.api.update_settings(&settings) {
                Ok(_) => self.toast("Ayarlar kaydedildi", false),
                Err(e) => self.set_error(e),
            }
        }

        ui.add_space(18.0);
        ui.separator();
        ui.label(egui::RichText::new("Yedekleme").strong());
        ui.label(
            egui::RichText::new("Sunucudaki veritabanı ve tüm resimleri ZIP olarak indirir.")
                .small()
                .weak(),
        );
        if ui.button("💾 Sunucu Yedeğini İndir (ZIP)").clicked() {
            self.download_backup();
        }
    }

    /// Doğrulama sekmesi (yalnızca yönetici): form girdi kuralları.
    fn settings_validation_tab(&mut self, ui: &mut egui::Ui) {
        if !self.is_admin() {
            ui.label("Bu sekme yalnızca yöneticiye açıktır.");
            return;
        }
        ui.label(egui::RichText::new("Form Doğrulama Kuralları (Validasyon)").strong());
        ui.label(
            egui::RichText::new(
                "Portföy formunda zorunlu alanlar ve sınır değerler. Kurallar sunucuda saklanır ve tüm kullanıcılar için geçerlidir.",
            )
            .small()
            .weak(),
        );
        ui.separator();
        egui::Grid::new("val_grid").num_columns(2).spacing([16.0, 8.0]).show(ui, |ui| {
            ui.label("Başlık en az kaç karakter");
            ui.add(egui::DragValue::new(&mut self.settings.val_title_min).range(1..=200));
            ui.end_row();
            ui.label("Açıklama en az kaç karakter (0 = zorunlu değil)");
            ui.add(egui::DragValue::new(&mut self.settings.val_desc_min).range(0..=2000));
            ui.end_row();
            ui.label("En düşük fiyat");
            ui.add(egui::DragValue::new(&mut self.settings.val_price_min).speed(1000.0).max_decimals(0));
            ui.end_row();
            ui.label("En düşük alan (m²)");
            ui.add(egui::DragValue::new(&mut self.settings.val_area_min).speed(1.0).max_decimals(0));
            ui.end_row();
            ui.label("Telefonda en az kaç rakam (0 = kontrol yok)");
            ui.add(egui::DragValue::new(&mut self.settings.val_phone_digits).range(0..=15));
            ui.end_row();
            ui.label("Sahip telefonu zorunlu olsun");
            ui.checkbox(&mut self.settings.val_phone_required, "Zorunlu");
            ui.end_row();
            ui.label("Sahip e-postası zorunlu olsun");
            ui.checkbox(&mut self.settings.val_email_required, "Zorunlu");
            ui.end_row();
        });
        ui.add_space(12.0);
        if ui.button("Doğrulama Kurallarını Kaydet").clicked() {
            let settings = self.settings.clone();
            match self.api.update_settings(&settings) {
                Ok(_) => self.toast("Doğrulama kuralları kaydedildi", false),
                Err(e) => self.set_error(e),
            }
        }
    }

    /// Kullanıcılar sekmesi (yalnızca yönetici).
    fn settings_users_tab(&mut self, ui: &mut egui::Ui) {
        if !self.is_admin() {
            ui.label("Bu sekme yalnızca yöneticiye açıktır.");
            return;
        }
        ui.label(egui::RichText::new("Kullanıcı Yönetimi ve Rol Atama").strong());
        ui.separator();
        self.users_manager_ui(ui);
    }

    fn download_backup(&mut self) {
        let dir = export_dir();
        let _ = std::fs::create_dir_all(&dir);
        match self.api.download_backup() {
            Ok(bytes) => {
                let name = format!(
                    "gayrimenkul360-yedek-{}.zip",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                );
                let path = dir.join(name);
                match std::fs::write(&path, &bytes) {
                    Ok(_) => {
                        self.toast(format!("Yedek kaydedildi: {}", path.display()), false);
                        let _ = open::that(&path);
                    }
                    Err(e) => self.set_error(format!("Yedek yazılamadı: {e}")),
                }
            }
            Err(e) => self.set_error(e),
        }
    }

    fn profile_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("Profilim");
        ui.separator();
        let photo = self.user.as_ref().and_then(|u| u.photo.clone());

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                if let Some(name) = &photo {
                    if let Some(tex) = self.texture(ctx, name) {
                        ui.add(egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(120.0, 120.0))));
                    }
                } else {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(120.0, 120.0), egui::Sense::hover());
                    ui.painter_at(rect).rect_filled(rect, 8.0, egui::Color32::from_gray(70));
                }
                if ui.button("Profil Fotoğrafı Seç").clicked() {
                    if let Some(files) = pick_files() {
                        if let Some(f) = files.into_iter().next() {
                            match self.api.upload_my_photo(&f) {
                                Ok(u) => {
                                    if let Some(p) = &u.photo {
                                        self.textures.remove(p);
                                    }
                                    self.user = Some(u);
                                    self.toast("Fotoğraf güncellendi", false);
                                }
                                Err(e) => self.set_error(e),
                            }
                        }
                    }
                }
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                egui::Grid::new("profile_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                    ui.label("Ad Soyad");
                    let before = self.profile_name.clone();
                    ui.add(egui::TextEdit::singleline(&mut self.profile_name).desired_width(280.0));
                    if self.profile_name != before {
                        self.profile_name = tr_title_case(&self.profile_name);
                    }
                    ui.end_row();
                    ui.label("Telefon");
                    ui.add(egui::TextEdit::singleline(&mut self.profile_phone).desired_width(280.0));
                    ui.end_row();
                    ui.label("E-posta");
                    ui.add(egui::TextEdit::singleline(&mut self.profile_email).desired_width(280.0));
                    ui.end_row();
                });
                if ui.button("Bilgileri Kaydet").clicked() {
                    let input = UpdateProfileRequest {
                        full_name: self.profile_name.clone(),
                        phone: self.profile_phone.clone(),
                        email: self.profile_email.clone(),
                    };
                    match self.api.update_profile(&input) {
                        Ok(u) => { self.user = Some(u); self.toast("Profil güncellendi", false); }
                        Err(e) => self.set_error(e),
                    }
                }
            });
        });

        ui.add_space(16.0);
        ui.separator();
        ui.label(egui::RichText::new("Parola Değiştir").strong());
        egui::Grid::new("pass_grid").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
            ui.label("Mevcut Parola");
            ui.add(egui::TextEdit::singleline(&mut self.old_password).password(true).desired_width(240.0));
            ui.end_row();
            ui.label("Yeni Parola");
            ui.add(egui::TextEdit::singleline(&mut self.new_password).password(true).desired_width(240.0));
            ui.end_row();
        });
        if ui.button("Parolayı Değiştir").clicked() {
            match self.api.change_password(&self.old_password, &self.new_password) {
                Ok(_) => {
                    self.toast("Parola değiştirildi", false);
                    self.old_password.clear();
                    self.new_password.clear();
                }
                Err(e) => self.set_error(e),
            }
        }
    }
}

impl FormState {
    fn current_mahalles(&self) -> Vec<&'static str> {
        if self.ilce == 0 {
            Vec::new()
        } else {
            common::mugla::neighborhoods(common::mugla::DISTRICTS[self.ilce - 1]).to_vec()
        }
    }
}

/// Açılır liste yardımcısı.
/// - `extra` boşsa: `idx` doğrudan `items` içine 0 tabanlı indekstir.
/// - `extra` doluysa: `idx == 0` için `extra[0]` (ör. "Seçiniz"), aksi halde `items[idx-1]` gösterilir.
fn combo_index(ui: &mut egui::Ui, id: &str, idx: &mut usize, extra: &[&str], items: &[&str]) {
    let selected_text = if extra.is_empty() {
        items.get((*idx).min(items.len().saturating_sub(1))).copied().unwrap_or("").to_string()
    } else if *idx == 0 {
        extra[0].to_string()
    } else {
        items.get(idx.saturating_sub(1)).copied().unwrap_or("").to_string()
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            if extra.is_empty() {
                for (i, item) in items.iter().enumerate() {
                    ui.selectable_value(idx, i, *item);
                }
            } else {
                ui.selectable_value(idx, 0, extra[0]);
                for (i, item) in items.iter().enumerate() {
                    ui.selectable_value(idx, i + 1, *item);
                }
            }
        });
}

fn detail_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.label(egui::RichText::new(label).strong());
    ui.label(if value.is_empty() { "-" } else { value });
    ui.end_row();
}

fn format_price(v: f64, currency: &str) -> String {
    let n = v.round() as i64;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    let r: String = out.chars().rev().collect();
    let sym = match currency {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        _ => "₺",
    };
    format!("{r} {sym}")
}

fn shift_duration(start: &str, end: Option<&str>) -> String {
    let parse = |s: &str| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok();
    if let (Some(a), Some(b)) = (parse(start), end.and_then(parse)) {
        let d = b - a;
        let mins = d.num_minutes();
        format!("{} sa {} dk", mins / 60, mins % 60)
    } else {
        "-".to_string()
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

pub fn export_dir() -> PathBuf {
    let base = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join("Gayrimenkul360-Cikti")
}

/// Basit yerel dosya seçici (GTK gerektirmez). Kullanıcı tam yolu girer.
/// Sürükle-bırak da desteklenir; bu fonksiyon ek yöntemdir.
fn pick_files() -> Option<Vec<PathBuf>> {
    use std::io::Write;
    // Windows'ta PowerShell, Linux'ta zenity/kdialog varsa kullan, yoksa basit konsol yolu.
    #[cfg(target_os = "windows")]
    {
        let script = r#"Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Multiselect=$true; $d.Filter='Resimler|*.jpg;*.jpeg;*.png;*.webp;*.gif;*.bmp'; if($d.ShowDialog() -eq 'OK'){ $d.FileNames | ForEach-Object { Write-Output $_ } }"#;
        if let Ok(out) = std::process::Command::new("powershell").args(["-NoProfile", "-Command", script]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            let files: Vec<PathBuf> = text.lines().filter(|l| !l.trim().is_empty()).map(PathBuf::from).collect();
            if !files.is_empty() {
                return Some(files);
            }
        }
        return None;
    }
    #[cfg(not(target_os = "windows"))]
    {
        // zenity veya kdialog dene
        let attempts: [(&str, Vec<&str>); 2] = [
            ("zenity", vec!["--file-selection", "--multiple", "--separator=\n", "--file-filter=Resimler | *.jpg *.jpeg *.png *.webp *.gif *.bmp"]),
            ("kdialog", vec!["--getopenfilename", ".", "Images (*.jpg *.jpeg *.png *.webp *.gif *.bmp)"]),
        ];
        for (cmd, args) in attempts {
            if let Ok(out) = std::process::Command::new(cmd).args(&args).output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let files: Vec<PathBuf> = text.lines().filter(|l| !l.trim().is_empty()).map(PathBuf::from).collect();
                    if !files.is_empty() {
                        return Some(files);
                    }
                }
            }
        }
        // Son çare: terminalden yol iste
        print!("Resim yolu (virgülle birden fazla): ");
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_ok() {
            let files: Vec<PathBuf> = line
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .map(PathBuf::from)
                .collect();
            if !files.is_empty() {
                return Some(files);
            }
        }
        None
    }
}
