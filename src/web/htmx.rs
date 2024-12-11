use std::fmt::Display;

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};

pub fn redirect(to: impl Display) -> Response<Body> {
    (
        StatusCode::MOVED_PERMANENTLY,
        [("HX-Redirect", to.to_string())],
        (),
    )
        .into_response()
}
