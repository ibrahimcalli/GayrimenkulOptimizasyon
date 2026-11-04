use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use common::{ApiError, Role, User};

use crate::state::{map_user, AppState};

pub struct CurrentUser(pub User);

pub struct AdminUser(pub User);

#[axum::async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let token = match token {
            Some(t) if !t.is_empty() => t,
            _ => return Err(unauthorized("Oturum bulunamadı")),
        };

        let conn = state.db.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT u.* FROM users u JOIN sessions s ON s.user_id = u.id
                 WHERE s.token = ?1 AND u.active = 1",
            )
            .map_err(|e| internal(&e.to_string()))?;
        let user = stmt
            .query_row([token], map_user)
            .map_err(|_| unauthorized("Geçersiz veya süresi dolmuş oturum"))?;
        Ok(CurrentUser(user))
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AdminUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
        if user.role != Role::Admin {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ApiError {
                    error: "Bu işlem için yönetici yetkisi gerekli".into(),
                }),
            )
                .into_response());
        }
        Ok(AdminUser(user))
    }
}

pub fn unauthorized(msg: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(ApiError { error: msg.into() }),
    )
        .into_response()
}

pub fn internal(msg: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError { error: msg.into() }),
    )
        .into_response()
}

pub fn bad_request(msg: &str) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { error: msg.into() }),
    )
        .into_response()
}

pub fn not_found(msg: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ApiError { error: msg.into() }),
    )
        .into_response()
}
