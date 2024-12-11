use axum::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    id: i64,
    pub username: String,
    password: String,
}

impl User {
    pub fn from_env() -> Self {
        Self {
            id: 1,
            username: std::env::var("USER_NAME").unwrap(),
            password: std::env::var("USER_PASSWORD").unwrap(),
        }
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password.as_bytes() // We use the password hash as the auth
                                 // hash--what this means
                                 // is when the user changes their password the
                                 // auth session becomes invalid.
    }
}

#[derive(Debug, Clone)]
pub struct Backend {
    user: User,
}

impl Backend {
    pub fn new(user: User) -> Self {
        Self { user }
    }
}

#[derive(Debug)]
pub struct AuthError(anyhow::Error);

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl std::error::Error for AuthError {}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        if creds.username == self.user.username && creds.password == self.user.password {
            Ok(Some(self.user.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        if *user_id == self.user.id {
            Ok(Some(self.user.clone()))
        } else {
            Ok(None)
        }
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;

// This allows us to extract the authentication fields from forms. We use this
// to authenticate requests with the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub next: Option<String>,
}
