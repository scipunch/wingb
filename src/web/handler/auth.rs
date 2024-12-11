use crate::web::app_state::AppState;
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(post::login))
        .route("/login", get(get::login))
        .route("/logout", get(get::logout))
}

mod post {
    use axum::http::StatusCode;
    use tracing::{info, warn};

    use crate::web::{
        auth::{AuthSession, Credentials},
        htmx,
    };

    use super::*;

    pub async fn login(
        mut auth_session: AuthSession,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        info!("Logging in");
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                warn!("Wrong credentials: {:?}", creds);
                let mut login_url = "/login".to_string();
                if let Some(next) = creds.next {
                    login_url = format!("{}?next={}", login_url, next);
                };
                return htmx::redirect(&login_url);
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let to = if let Some(ref next) = creds.next {
            next
        } else {
            "/"
        };
        htmx::redirect(to)
    }
}

mod get {
    use askama::Template;
    use axum::{http::StatusCode, response::Redirect};

    use crate::web::auth::AuthSession;

    use super::*;

    #[derive(Template)]
    #[template(path = "page/login.html")]
    struct Login {}

    pub async fn login() -> impl IntoResponse {
        Login {}
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
