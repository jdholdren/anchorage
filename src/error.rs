use std::fmt::{Debug, Display};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize};

use crate::ReqContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Kind {
    Permission,
    BadRequest,
    Internal,
    NotFound,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "{:?}", self)
    }
}

/// Error type that we can use everywhere, should provide some documentation
/// on what exactly you're trying to do and where it failed.
#[derive(Debug, Deserialize)]
pub struct Error {
    pub user: String,              // What user was trying to operate
    pub op: String,                // What operation you were trying to do
    pub kind: Kind,                // What kind of error this is
    pub inner_err: Option<String>, // The inner error
}

impl Display for Error {
    // Should look like:
    // { user: "blah-blah", op: "server.Put", etc }
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "{{ ")?;

        write!(w, "user: '{}', ", self.user)?;
        write!(w, "op: '{}', ", self.op)?;
        write!(w, "kind: '{}', ", self.kind)?;

        if let Some(err) = &self.inner_err {
            write!(w, "err: '{:?}'", err)?;
        } else {
            write!(w, "err: nil")?;
        }

        write!(w, " }}")
    }
}

impl std::error::Error for Error {}

// Custom serialization for serde to handle error trait object
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Error", 4)?;
        s.serialize_field("user", &self.user)?;
        s.serialize_field("op", &self.op)?;
        s.serialize_field("kind", &self.kind.to_string())?;
        if let Some(err) = &self.inner_err {
            s.serialize_field("err", &format!("{:?}", err))?;
        }
        s.end()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status_code = match self.kind {
            Kind::Permission => StatusCode::FORBIDDEN,
            Kind::BadRequest => StatusCode::BAD_REQUEST,
            Kind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Kind::NotFound => StatusCode::NOT_FOUND,
        };

        (status_code, Json(self)).into_response()
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self {
            op: value
                .url()
                .map(|url| url.path().to_owned())
                .unwrap_or("unspecified".to_owned()),
            user: String::new(),
            kind: Kind::Internal,
            inner_err: Some(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_formatted() {
        let err: Box<dyn std::error::Error> = Box::new(Error {
            user: "Bilbo".to_string(),
            op: String::from("server.Put"),
            kind: Kind::Permission,
            inner_err: Some("inner error".to_owned()),
        });

        let formatted = format!("{}", err);
        assert_eq!(
            formatted,
            "{ user: 'Bilbo', op: 'server.Put', kind: 'Permission', err: 'inner error' }",
        );
    }
}

pub trait WithReqContext<T> {
    fn with_ctx(self, ctx: &ReqContext, kind: Kind) -> Result<T, Error>;
}

impl<T, E: Display> WithReqContext<T> for Result<T, E> {
    fn with_ctx(self, ctx: &ReqContext, kind: Kind) -> Result<T, Error> {
        self.map_err(|err| Error {
            op: ctx.op.clone(),
            user: ctx.user.clone(),
            kind,
            inner_err: Some(err.to_string()),
        })
    }
}
