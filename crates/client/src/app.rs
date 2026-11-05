use std::collections::HashMap;
use std::path::PathBuf;

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
    pub area_m2: f64,
    pub room_count: String,
    pub building_age: String,
    pub floor: String,
    pub total_floors: String,
    pub heating_choice: usize,
    pub owner_type_choice: usize,
    pub furnished: bool,
    pub ilce: usize,    // 0 = seçilmedi, diğer: DISTRICTS[i-1]
    pub mahalle: usize, // 0 = seçilmedi
    pub address: String,
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
    pub work_days: i64,
    pub daily_hours: f64,
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
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    View,
    Output,
    Validation,
    Users,
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
        Self::install_turkish_fonts(&cc.egui_ctx);
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
        };
        app.apply_theme(&cc.egui_ctx);
        cc.egui_ctx.set_zoom_factor(app.zoom as f32);
        app
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
        if self.is_admin() {
            self.load_crm();
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
            1 => Some("yayinda"),
            2 => Some("kiralandi"),
            3 => Some("satildi"),
            4 => Some("taslak"),
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

    /// Türkçe karakter kümesini (ç ğ ı ö ş ü İ Ğ Ş) tam destekleyen font kurulumu.
/// Roboto tüm metinlerde öncelikli kullanılır; emoji/ikon fontları yedekte kalır.
fn install_turkish_fonts(ctx: &egui::Context) {
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

        egui::CentralPanel::default().show(ctx, |ui| {
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

        self.messages(ctx);
        self.notification_popup(ctx);
        self.cover_zoom_window(ctx);
    }
}

// =====================================================================
//  LOGIN
// =====================================================================

impl App {
    fn login_screen(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.heading(egui::RichText::new("GAYRİMENKUL360").size(34.0).strong());
            ui.label("Ofis Portföy ve Müşteri Yönetim Sistemi");
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
                        egui::RichText::new("İlk giriş: admin / admin123  (sonra değiştirin)")
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
                ui.heading(egui::RichText::new("GAYRİMENKUL360").color(egui::Color32::from_rgb(46, 116, 181)));
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
                nav_button(ui, &mut target, self.screen, Screen::Welcome, "🏠  Portföyler");
                nav_button(ui, &mut target, self.screen, Screen::PortfolioForm, "➕  Yeni Portföy");
                nav_button(ui, &mut target, self.screen, Screen::Tasks, "✅  Görevler");
                nav_button(ui, &mut target, self.screen, Screen::Goals, "🎯  Hedefler");
                nav_button(ui, &mut target, self.screen, Screen::Shifts, "🕒  Mesai");
                ui.separator();
                if self.is_admin() {
                    nav_button(ui, &mut target, self.screen, Screen::Crm, "👥  CRM Kayıtları");
                    nav_button(ui, &mut target, self.screen, Screen::Users, "🧑‍💼  Kullanıcılar");
                    ui.separator();
                }
                nav_button(ui, &mut target, self.screen, Screen::Settings, "⚙  Ayarlar");
                nav_button(ui, &mut target, self.screen, Screen::Profile, "👤  Profilim");
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
                self.form = FormState::default();
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

fn nav_button(ui: &mut egui::Ui, target: &mut Option<Screen>, current: Screen, to: Screen, label: &str) {
    let selected = current == to;
    if ui
        .add_sized(
            [ui.available_width(), 30.0],
            egui::SelectableLabel::new(selected, label),
        )
        .clicked()
    {
        *target = Some(to);
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct ClientConfig {
    server_url: String,
    #[serde(default)]
    theme: String,
    #[serde(default)]
    zoom: f64,
}

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
                &["Yayında", "Kiralandı", "Satıldı", "Taslak"],
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
                combo_index(ui, "pf_st", &mut self.form.status, &[], &["Taslak", "Yayında", "Kiralandı", "Satıldı"]);
                ui.end_row();

                ui.label("Fiyat");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.form.price).speed(1000.0).max_decimals(0));
                    combo_index(ui, "pf_cur", &mut self.form.currency, &[], &CURRENCIES);
                });
                ui.end_row();

                ui.label("Alan (m²)");
                ui.add(egui::DragValue::new(&mut self.form.area_m2).speed(1.0).max_decimals(0));
                ui.end_row();

                ui.label("Oda Sayısı");
                ui.add(egui::TextEdit::singleline(&mut self.form.room_count).desired_width(140.0).hint_text("3+1"));
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

                ui.label("Isıtma");
                combo_index(ui, "pf_heat", &mut self.form.heating_choice, &["Seçiniz"], &HEATINGS);
                ui.end_row();

                ui.label("Eşyalı");
                ui.checkbox(&mut self.form.furnished, "Eşyalı");
                ui.end_row();

                ui.label("İl");
                ui.label("Muğla (sabit)");
                ui.end_row();

                ui.label("İlçe *");
                combo_index(ui, "pf_ilce", &mut self.form.ilce, &["Seçiniz"], common::mugla::DISTRICTS);
                ui.end_row();

                ui.label("Mahalle *");
                let mahs = self.form.current_mahalles();
                combo_index(ui, "pf_mah", &mut self.form.mahalle, &["Seçiniz"], &mahs);
                ui.end_row();

                ui.label("Açık Adres");
                ui.add(egui::TextEdit::singleline(&mut self.form.address).desired_width(360.0));
                ui.end_row();

                ui.label("Konum (Enlem, Boylam)");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.form.latitude).speed(0.0001).max_decimals(6).prefix("En: "));
                    ui.add(egui::DragValue::new(&mut self.form.longitude).speed(0.0001).max_decimals(6).prefix("Boy: "));
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
                    self.form = FormState::default();
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

        let input = PortfolioInput {
            title: self.form.title.trim().to_string(),
            listing_type: ListingType::ALL[self.form.listing_type.min(1)].as_str().to_string(),
            category: PropertyCategory::ALL[self.form.category.min(4)].as_str().to_string(),
            status: PortfolioStatus::ALL[self.form.status.min(3)].as_str().to_string(),
            price: self.form.price,
            currency: CURRENCIES[self.form.currency.min(3)].to_string(),
            area_m2: self.form.area_m2,
            room_count: self.form.room_count.clone(),
            building_age: self.form.building_age.clone(),
            floor: self.form.floor.clone(),
            total_floors: self.form.total_floors.clone(),
            heating: if self.form.heating_choice == 0 {
                String::new()
            } else {
                HEATINGS[(self.form.heating_choice - 1).min(HEATINGS.len() - 1)].to_string()
            },
            furnished: self.form.furnished,
            il: "Muğla".to_string(),
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
                self.form = FormState::default();
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
                        if let Some(tex) = self.texture(ctx, name) {
                            let resp = ui.add(
                                egui::Image::new(egui::load::SizedTexture::new(tex.id(), egui::vec2(180.0, 135.0)))
                                    .sense(egui::Sense::click()),
                            );
                            if resp.clicked() {
                                self.cover_zoom = Some((name.clone(), 1.0));
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
                detail_row(ui, "Alan", &format!("{} m²", portfolio.area_m2));
                detail_row(ui, "Oda", &portfolio.room_count);
                detail_row(ui, "Bina Yaşı", &portfolio.building_age);
                detail_row(ui, "Kat", &format!("{} / {}", portfolio.floor, portfolio.total_floors));
                detail_row(ui, "Isıtma", &portfolio.heating);
                detail_row(ui, "Eşyalı", if portfolio.furnished { "Evet" } else { "Hayır" });
                detail_row(ui, "Konum", &format!("{} / {} / {}", portfolio.il, portfolio.ilce, portfolio.mahalle));
                detail_row(ui, "Adres", &portfolio.address);
                if portfolio.latitude != 0.0 || portfolio.longitude != 0.0 {
                    detail_row(ui, "Koordinat", &format!("{:.5}, {:.5}", portfolio.latitude, portfolio.longitude));
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

    fn start_edit(&mut self, p: &Portfolio) {
        let ilce = common::mugla::DISTRICTS.iter().position(|d| *d == p.ilce).map(|i| i + 1).unwrap_or(0);
        let mahs = common::mugla::neighborhoods(&p.ilce);
        let mahalle = mahs.iter().position(|m| *m == p.mahalle).map(|i| i + 1).unwrap_or(0);
        let listing = match p.listing_type.as_str() { "kiralik" => 1, _ => 0 };
        let category = match p.category.as_str() {
            "isyeri" => 1, "arsa" => 2, "tarla" => 3, "turizm" => 4, _ => 0,
        };
        let status = match p.status.as_str() {
            "yayinda" => 1, "kiralandi" => 2, "satildi" => 3, _ => 0,
        };
        let currency = CURRENCIES.iter().position(|c| *c == p.currency).unwrap_or(0);
        let heating_choice = HEATINGS.iter().position(|h| *h == p.heating).map(|i| i + 1).unwrap_or(0);
        let owner_choice = if p.owner_crm_id.is_some() && self.is_admin() {
            self.crm.iter().position(|c| Some(c.id) == p.owner_crm_id).map(|i| i + 2).unwrap_or(0)
        } else {
            0
        };
        self.form = FormState {
            edit_id: Some(p.id),
            title: p.title.clone(),
            listing_type: listing,
            category,
            status,
            price: p.price,
            currency,
            area_m2: p.area_m2,
            room_count: p.room_count.clone(),
            building_age: p.building_age.clone(),
            floor: p.floor.clone(),
            total_floors: p.total_floors.clone(),
            heating_choice,
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
                        ui.add(egui::TextEdit::singleline(&mut self.task_form.due_date).desired_width(120.0).hint_text("2026-11-08"));
                        ui.add(egui::TextEdit::singleline(&mut self.task_form.due_time).desired_width(80.0).hint_text("16:00"));
                    });
                    ui.end_row();
                    ui.label("Öncelik");
                    combo_index(ui, "task_prio", &mut self.task_form.priority, &[], &PRIORITIES);
                    ui.end_row();
                });
                if ui.button("Görevi Ata").clicked() {
                    let due = if self.task_form.due_date.trim().is_empty() {
                        String::new()
                    } else if self.task_form.due_time.trim().is_empty() {
                        self.task_form.due_date.clone()
                    } else {
                        format!("{} {}", self.task_form.due_date, self.task_form.due_time)
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
                            ui.add(egui::TextEdit::singleline(&mut self.goal_form.period_start).desired_width(120.0).hint_text("2026-11-01"));
                            ui.add(egui::TextEdit::singleline(&mut self.goal_form.period_end).desired_width(120.0).hint_text("2026-11-30"));
                        });
                        ui.end_row();
                    });
                    if ui.button("Hedefi Ata").clicked() {
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
                            period_start: self.goal_form.period_start.clone(),
                            period_end: self.goal_form.period_end.clone(),
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

        let tabs = [("📋 Vardiya Takvimi", 0usize), ("✅ Mesai Onay Listesi", 1usize)];
        ui.horizontal(|ui| {
            for (label, i) in tabs {
                let admin_only = i == 1;
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
            _ => self.shift_approval_tab(ui),
        }
    }

    /// Salt-okunur vardiya takvimi: kullanıcılar kendi ve ekip arkadaşlarının
    /// tanımlı mesai saatlerini görür; düzenleme yalnızca yöneticide.
    fn shift_plans_tab(&mut self, ui: &mut egui::Ui) {
        if self.is_admin() {
            egui::CollapsingHeader::new("➕ Vardiya Planı Ata / Güncelle").show(ui, |ui| {
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
                    ui.label("Aylık iş günü");
                    ui.add(egui::DragValue::new(&mut self.plan_form.work_days).range(1..=31).suffix(" gün"));
                    ui.end_row();
                    ui.label("Günlük çalışma saati");
                    ui.add(egui::DragValue::new(&mut self.plan_form.daily_hours).range(0.5..=16.0).speed(0.5).suffix(" sa"));
                    ui.end_row();
                    ui.label("Açılış / Kapanış saati");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.plan_form.start_time).desired_width(80.0).hint_text("09:00"));
                        ui.label("—");
                        ui.add(egui::TextEdit::singleline(&mut self.plan_form.end_time).desired_width(80.0).hint_text("18:00"));
                    });
                    ui.end_row();
                    ui.label("Not");
                    ui.add(egui::TextEdit::singleline(&mut self.plan_form.note).desired_width(240.0));
                    ui.end_row();
                });
                if ui.button("Planı Kaydet").clicked() {
                    if let Some(u) = self.users.get(self.plan_form.user_idx) {
                        let input = ShiftPlanInput {
                            user_id: u.id,
                            month: self.plan_form.month.trim().to_string(),
                            work_days: self.plan_form.work_days,
                            daily_hours: self.plan_form.daily_hours,
                            start_time: if self.plan_form.start_time.is_empty() { "09:00".into() } else { self.plan_form.start_time.clone() },
                            end_time: if self.plan_form.end_time.is_empty() { "18:00".into() } else { self.plan_form.end_time.clone() },
                            note: self.plan_form.note.clone(),
                        };
                        match self.api.upsert_shift_plan(&input) {
                            Ok(_) => {
                                self.toast("Vardiya planı kaydedildi", false);
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

        egui::Grid::new("plans_list").num_columns(7).striped(true).spacing([16.0, 6.0]).show(ui, |ui| {
            for h in ["Personel", "Ay", "Aylık Gün", "Günlük Saat", "Açılış", "Kapanış", "Not"] {
                ui.label(egui::RichText::new(h).strong());
            }
            ui.end_row();
            for p in &self.plans {
                ui.label(&p.user_name);
                ui.label(&p.month);
                ui.label(format!("{} gün", p.work_days));
                ui.label(format!("{:.1} saat", p.daily_hours));
                ui.label(&p.start_time);
                ui.label(&p.end_time);
                ui.label(egui::RichText::new(&p.note).small().weak());
                ui.end_row();
            }
        });
        if self.plans.is_empty() {
            ui.label(egui::RichText::new("Henüz vardiya planı tanımlanmadı.").small().weak());
        }
    }

    /// Yönetici: gerçekleşen mesai gün/saat kaydı girer ve onaylar.
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
                ui.add(egui::TextEdit::singleline(&mut self.att_form.date).desired_width(120.0).hint_text("2026-11-08"));
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
                    let input = ShiftAttendanceInput {
                        user_id: u.id,
                        date: self.att_form.date.trim().to_string(),
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
                } else {
                    self.set_error("Önce kullanıcı seçin".into());
                }
            }
        });
        ui.separator();

        ui.label(egui::RichText::new("Onay kutucuğunu işaretleyerek mesaiyi onaylayın:").strong());
        let mut toggle: Option<i64> = None;
        egui::Grid::new("att_list").num_columns(5).striped(true).spacing([16.0, 6.0]).show(ui, |ui| {
            for h in ["Personel", "Tarih", "Saat", "Onay", "Not"] {
                ui.label(egui::RichText::new(h).strong());
            }
            ui.end_row();
            for a in &self.attendance {
                ui.label(&a.user_name);
                ui.label(&a.date);
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
                    ui.add(egui::TextEdit::singleline(&mut self.crm_form.full_name).desired_width(300.0));
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

        let tabs: [(SettingsTab, &str); 4] = [
            (SettingsTab::View, "🎨 Görünüm"),
            (SettingsTab::Output, "📄 Çıktı"),
            (SettingsTab::Validation, "✅ Doğrulama"),
            (SettingsTab::Users, "🧑‍💼 Kullanıcılar"),
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
        }
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
                    self.save_config();
                }
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
                    ui.add(egui::TextEdit::singleline(&mut self.profile_name).desired_width(280.0));
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
