use std::path::Path;
use std::time::Duration;

use common::discovery::{DiscoveryReply, DiscoveredServer, REQUEST, UDP_PORT};
use common::*;
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::Method;
use serde::de::DeserializeOwned;

pub type ApiResult<T> = Result<T, String>;

#[derive(Clone)]
pub struct Api {
    pub base: String,
    pub token: Option<String>,
    client: Client,
}

impl Api {
    pub fn new(base: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());
        Self {
            base: normalize_url(&base),
            token: None,
            client,
        }
    }

    pub fn set_base(&mut self, base: &str) {
        self.base = normalize_url(base);
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    fn req(&self, method: Method, path: &str) -> RequestBuilder {
        let mut rb = self.client.request(method, self.url(path));
        if let Some(t) = &self.token {
            rb = rb.bearer_auth(t);
        }
        rb
    }

    fn handle<T: DeserializeOwned>(resp: Response) -> ApiResult<T> {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        if status.is_success() {
            if text.trim().is_empty() {
                return serde_json::from_str("null").map_err(|e| e.to_string());
            }
            serde_json::from_str(&text).map_err(|e| format!("Sunucu yanıtı okunamadı: {e}"))
        } else {
            let msg = serde_json::from_str::<ApiError>(&text)
                .map(|e| e.error)
                .unwrap_or_else(|_| format!("Sunucu hatası ({status})"));
            Err(msg)
        }
    }

    fn handle_empty(resp: Response) -> ApiResult<()> {
        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = resp.text().unwrap_or_default();
            let msg = serde_json::from_str::<ApiError>(&text)
                .map(|e| e.error)
                .unwrap_or_else(|_| format!("Sunucu hatası ({status})"));
            Err(msg)
        }
    }

    // ---------------- auth ----------------
    pub fn login(&mut self, username: &str, password: &str) -> ApiResult<AuthResponse> {
        let resp = self
            .client
            .post(self.url("/api/auth/login"))
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .map_err(net_err)?;
        let auth: AuthResponse = Self::handle(resp)?;
        self.token = Some(auth.token.clone());
        Ok(auth)
    }

    pub fn logout(&self) {
        let _ = self
            .client
            .post(self.url("/api/auth/logout"))
            .bearer_auth(self.token.clone().unwrap_or_default())
            .send();
    }

    pub fn me(&self) -> ApiResult<User> {
        Self::handle(self.req(Method::GET, "/api/me").send().map_err(net_err)?)
    }

    pub fn update_profile(&self, r: &UpdateProfileRequest) -> ApiResult<User> {
        Self::handle(self.req(Method::PUT, "/api/me").json(r).send().map_err(net_err)?)
    }

    pub fn change_password(&self, old: &str, new: &str) -> ApiResult<()> {
        Self::handle_empty(
            self.req(Method::POST, "/api/me/password")
                .json(&ChangePasswordRequest {
                    old_password: old.to_string(),
                    new_password: new.to_string(),
                })
                .send()
                .map_err(net_err)?,
        )
    }

    pub fn upload_my_photo(&self, path: &Path) -> ApiResult<User> {
        let form = file_form("file", path)?;
        Self::handle(
            self.req(Method::POST, "/api/me/photo")
                .multipart(form)
                .send()
                .map_err(net_err)?,
        )
    }

    // ---------------- users ----------------
    pub fn list_users(&self) -> ApiResult<Vec<User>> {
        Self::handle(self.req(Method::GET, "/api/users").send().map_err(net_err)?)
    }

    pub fn create_user(&self, r: &CreateUserRequest) -> ApiResult<User> {
        Self::handle(self.req(Method::POST, "/api/users").json(r).send().map_err(net_err)?)
    }

    pub fn update_user(&self, id: i64, r: &CreateUserRequest) -> ApiResult<User> {
        Self::handle(
            self.req(Method::PUT, &format!("/api/users/{id}"))
                .json(r)
                .send()
                .map_err(net_err)?,
        )
    }

    pub fn toggle_user(&self, id: i64) -> ApiResult<User> {
        Self::handle(
            self.req(Method::POST, &format!("/api/users/{id}/toggle"))
                .send()
                .map_err(net_err)?,
        )
    }

    // ---------------- portfolio ----------------
    pub fn list_portfolios(&self, q: Option<&str>, ilce: Option<&str>, mahalle: Option<&str>, listing: Option<&str>, category: Option<&str>, status: Option<&str>, owner: Option<i64>, show_deleted: bool) -> ApiResult<Vec<Portfolio>> {
        let mut req = self.req(Method::GET, "/api/portfolios");
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(v) = q.filter(|s| !s.is_empty()) { params.push(("q", v.into())); }
        if let Some(v) = ilce.filter(|s| !s.is_empty()) { params.push(("ilce", v.into())); }
        if let Some(v) = mahalle.filter(|s| !s.is_empty()) { params.push(("mahalle", v.into())); }
        if let Some(v) = listing.filter(|s| !s.is_empty()) { params.push(("listing_type", v.into())); }
        if let Some(v) = category.filter(|s| !s.is_empty()) { params.push(("category", v.into())); }
        if let Some(v) = status.filter(|s| !s.is_empty()) { params.push(("status", v.into())); }
        if let Some(v) = owner { params.push(("owner", v.to_string())); }
        if show_deleted { params.push(("show_deleted", "1".into())); }
        if !params.is_empty() { req = req.query(&params); }
        Self::handle(req.send().map_err(net_err)?)
    }

    pub fn get_portfolio(&self, id: i64) -> ApiResult<Portfolio> {
        Self::handle(self.req(Method::GET, &format!("/api/portfolios/{id}")).send().map_err(net_err)?)
    }

    pub fn create_portfolio(&self, input: &PortfolioInput) -> ApiResult<Portfolio> {
        Self::handle(self.req(Method::POST, "/api/portfolios").json(input).send().map_err(net_err)?)
    }

    pub fn update_portfolio(&self, id: i64, input: &PortfolioInput) -> ApiResult<Portfolio> {
        Self::handle(
            self.req(Method::PUT, &format!("/api/portfolios/{id}"))
                .json(input)
                .send()
                .map_err(net_err)?,
        )
    }

    pub fn delete_portfolio(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::DELETE, &format!("/api/portfolios/{id}")).send().map_err(net_err)?)
    }

    pub fn restore_portfolio(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::POST, &format!("/api/portfolios/{id}/restore")).send().map_err(net_err)?)
    }

    pub fn upload_portfolio_images(&self, id: i64, paths: &[std::path::PathBuf]) -> ApiResult<Vec<String>> {
        let mut form = Form::new();
        for p in paths {
            let bytes = std::fs::read(p).map_err(|e| format!("Dosya okunamadı: {e}"))?;
            let name = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "resim.jpg".into());
            form = form.part("file", Part::bytes(bytes).file_name(name));
        }
        Self::handle(
            self.req(Method::POST, &format!("/api/portfolios/{id}/images"))
                .multipart(form)
                .send()
                .map_err(net_err)?,
        )
    }

    pub fn delete_portfolio_image(&self, id: i64, filename: &str) -> ApiResult<()> {
        Self::handle_empty(
            self.req(Method::DELETE, &format!("/api/portfolios/{id}/images/{filename}"))
                .send()
                .map_err(net_err)?,
        )
    }

    // ---------------- crm ----------------
    pub fn list_crm(&self, show_deleted: bool) -> ApiResult<Vec<CrmRecord>> {
        let mut req = self.req(Method::GET, "/api/crm");
        if show_deleted {
            req = req.query(&[("show_deleted", "1")]);
        }
        Self::handle(req.send().map_err(net_err)?)
    }
    pub fn create_crm(&self, r: &CrmInput) -> ApiResult<CrmRecord> {
        Self::handle(self.req(Method::POST, "/api/crm").json(r).send().map_err(net_err)?)
    }
    pub fn update_crm(&self, id: i64, r: &CrmInput) -> ApiResult<CrmRecord> {
        Self::handle(self.req(Method::PUT, &format!("/api/crm/{id}")).json(r).send().map_err(net_err)?)
    }
    pub fn delete_crm(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::DELETE, &format!("/api/crm/{id}")).send().map_err(net_err)?)
    }
    pub fn restore_crm(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::POST, &format!("/api/crm/{id}/restore")).send().map_err(net_err)?)
    }
    pub fn upload_crm_photo(&self, id: i64, path: &Path) -> ApiResult<CrmRecord> {
        let form = file_form("file", path)?;
        Self::handle(
            self.req(Method::POST, &format!("/api/crm/{id}/photo"))
                .multipart(form)
                .send()
                .map_err(net_err)?,
        )
    }

    // ---------------- tasks ----------------
    pub fn list_tasks(&self) -> ApiResult<Vec<Task>> {
        Self::handle(self.req(Method::GET, "/api/tasks").send().map_err(net_err)?)
    }
    pub fn create_task(&self, r: &TaskInput) -> ApiResult<Task> {
        Self::handle(self.req(Method::POST, "/api/tasks").json(r).send().map_err(net_err)?)
    }
    pub fn toggle_task(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::POST, &format!("/api/tasks/{id}/toggle")).send().map_err(net_err)?)
    }
    pub fn delete_task(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::DELETE, &format!("/api/tasks/{id}")).send().map_err(net_err)?)
    }

    // ---------------- goals ----------------
    pub fn list_goals(&self) -> ApiResult<Vec<Goal>> {
        Self::handle(self.req(Method::GET, "/api/goals").send().map_err(net_err)?)
    }
    pub fn create_goal(&self, r: &GoalInput) -> ApiResult<Goal> {
        Self::handle(self.req(Method::POST, "/api/goals").json(r).send().map_err(net_err)?)
    }
    pub fn delete_goal(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::DELETE, &format!("/api/goals/{id}")).send().map_err(net_err)?)
    }

    // ---------------- vardiya planı / mesai onay ----------------
    pub fn list_shift_plans(&self) -> ApiResult<Vec<ShiftPlan>> {
        Self::handle(self.req(Method::GET, "/api/shift/plans").send().map_err(net_err)?)
    }
    pub fn upsert_shift_plan(&self, r: &ShiftPlanInput) -> ApiResult<ShiftPlan> {
        Self::handle(self.req(Method::POST, "/api/shift/plan").json(r).send().map_err(net_err)?)
    }
    pub fn list_attendance(&self) -> ApiResult<Vec<ShiftAttendance>> {
        Self::handle(self.req(Method::GET, "/api/shift/attendance").send().map_err(net_err)?)
    }
    pub fn upsert_attendance(&self, r: &ShiftAttendanceInput) -> ApiResult<ShiftAttendance> {
        Self::handle(self.req(Method::POST, "/api/shift/attendance").json(r).send().map_err(net_err)?)
    }
    pub fn toggle_attendance(&self, id: i64) -> ApiResult<()> {
        Self::handle_empty(
            self.req(Method::POST, &format!("/api/shift/attendance/{id}/toggle-approve"))
                .send()
                .map_err(net_err)?,
        )
    }

    // ---------------- notifications ----------------
    pub fn list_notifications(&self) -> ApiResult<Vec<Notification>> {
        Self::handle(self.req(Method::GET, "/api/notifications").send().map_err(net_err)?)
    }
    pub fn read_all_notifications(&self) -> ApiResult<()> {
        Self::handle_empty(self.req(Method::POST, "/api/notifications/read-all").send().map_err(net_err)?)
    }

    // ---------------- settings ----------------
    pub fn get_settings(&self) -> ApiResult<OutputSettings> {
        Self::handle(self.req(Method::GET, "/api/settings").send().map_err(net_err)?)
    }
    pub fn update_settings(&self, s: &OutputSettings) -> ApiResult<OutputSettings> {
        Self::handle(self.req(Method::PUT, "/api/settings").json(s).send().map_err(net_err)?)
    }

    // ---------------- files ----------------
    pub fn download_file(&self, name: &str) -> ApiResult<Vec<u8>> {
        let resp = self
            .client
            .get(self.url(&format!("/api/files/{name}")))
            .send()
            .map_err(net_err)?;
        if resp.status().is_success() {
            let bytes = resp.bytes().map_err(|e| e.to_string())?;
            Ok(bytes.to_vec())
        } else {
            Err(format!("Dosya indirilemedi: {name}"))
        }
    }

    pub fn file_url(&self, name: &str) -> String {
        self.url(&format!("/api/files/{name}"))
    }

    // ---------------- yedekleme ----------------
    pub fn download_backup(&self) -> ApiResult<Vec<u8>> {
        let resp = self.req(Method::GET, "/api/backup").send().map_err(net_err)?;
        if resp.status().is_success() {
            Ok(resp.bytes().map_err(|e| e.to_string())?.to_vec())
        } else {
            let text = resp.text().unwrap_or_default();
            let msg = serde_json::from_str::<ApiError>(&text)
                .map(|e| e.error)
                .unwrap_or_else(|_| "Yedek alınamadı".to_string());
            Err(msg)
        }
    }

    // ---------------- bağlantı durumu ----------------
    pub fn health(&self) -> bool {
        self.client
            .get(self.url("/api/health"))
            .timeout(Duration::from_secs(4))
            .send()
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

/// Wi-Fi veya kablolu ağ üzerindeki Gayrimenkul360 sunucularını UDP yayını ile bulur.
/// `timeout` süresi boyunca cevapları toplar.
pub fn discover_servers(timeout: Duration) -> Vec<DiscoveredServer> {
    use std::net::UdpSocket;

    let mut found: Vec<DiscoveredServer> = Vec::new();
    let sock = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return found,
    };
    let _ = sock.set_broadcast(true);
    let _ = sock.set_read_timeout(Some(Duration::from_millis(300)));

    // Yayın adresine istek gönder (birden fazla ağ arayüzü için yaygın adresler denenir)
    let _ = sock.send_to(REQUEST, ("255.255.255.255", UDP_PORT));

    let start = std::time::Instant::now();
    let mut buf = [0u8; 512];
    while start.elapsed() < timeout {
        match sock.recv_from(&mut buf) {
            Ok((n, from)) => {
                if let Ok(reply) = serde_json::from_slice::<DiscoveryReply>(&buf[..n]) {
                    let server = DiscoveredServer {
                        ip: from.ip().to_string(),
                        port: reply.port,
                        name: reply.name,
                        version: reply.version,
                    };
                    if !found.iter().any(|f| f.url() == server.url()) {
                        found.push(server);
                    }
                }
            }
            Err(_) => {}
        }
    }
    found.sort_by(|a, b| a.name.cmp(&b.name));
    found
}

fn net_err(e: reqwest::Error) -> String {
    if e.is_connect() {
        format!("Sunucuya bağlanılamadı ({}). Sunucu bilgisayarı açık mı ve adres doğru mu?", e)
    } else if e.is_timeout() {
        "Sunucu yanıt vermedi (zaman aşımı). Ağ bağlantısını kontrol edin.".to_string()
    } else {
        format!("Ağ hatası: {e}")
    }
}

fn file_form(field: &str, path: &Path) -> ApiResult<Form> {
    let bytes = std::fs::read(path).map_err(|e| format!("Dosya okunamadı: {e}"))?;
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "dosya.jpg".into());
    Ok(Form::new().part(field.to_string(), Part::bytes(bytes).file_name(name)))
}

pub fn normalize_url(base: &str) -> String {
    let mut b = base.trim().trim_end_matches('/').to_string();
    if b.is_empty() {
        b = "http://127.0.0.1:8080".to_string();
    }
    if !b.starts_with("http://") && !b.starts_with("https://") {
        b = format!("http://{b}");
    }
    b
}
