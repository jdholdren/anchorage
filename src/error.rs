use std::fmt::{Debug, Display};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Deserializer};

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

/// InnerErr wraps an optional pointer to another error in a long chain of
/// errors. It can be None if the error you're looking at is the source of the error.
#[derive(Debug)]
pub struct InnerErr(pub Box<dyn std::error::Error>);

impl<'de> serde::Deserialize<'de> for InnerErr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        Ok(InnerErr(Box::new(StrError(s))))
    }
}

/// Error type that we can use everywhere, should provide some documentation
/// on what exactly you're trying to do and where it failed.
#[derive(Debug, Deserialize)]
pub struct Error {
    pub user: Option<String>,        // What user was trying to operate
    pub message: String,
    pub op: Option<String>,                // What operation you were trying to do
    pub kind: Kind,                // What kind of error this is
    pub inner_err: Option<InnerErr>, 
}

impl Error {
    // The common constructor of an Error from a generic other error
    pub fn from_err(msg: &str, err: Box<dyn std::error::Error>, kind: Kind) -> Self {
        Error {
            user: None,
            message: msg.to_owned(),
            inner_err: Some(InnerErr(err)),
            kind,
            op: None,
        }
    }
}

impl<T: std::error::Error> From<T> for Error {
    fn from(value: T) -> Self {
        Self {
            op: None,
            kind: Kind::Internal, // Just bad internal things by default
            message: value.to_string(),
            inner_err: None,
            user: None,
        }
    }
}

impl Display for Error {
    // Should look like:
    // { user: "blah-blah", op: "server.Put", etc }
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "{{ ")?;

        if let Some(user) = &self.user {
            write!(w, "user: '{:?}'", user)?;
        }
        if let Some(op) = &self.op {
        write!(w, "op: '{}', ", op)?;
        }
        write!(w, "kind: '{}', ", self.kind)?;

        if let Some(err) = &self.inner_err {
            write!(w, "err: '{:?}'", err)?;
        }

        write!(w, " }}")
    }
}

// impl std::error::Error for Error {}

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

#[derive(Debug)]
pub struct StrError(String);

impl std::error::Error for StrError {}

impl std::fmt::Display for StrError {
    fn fmt(&self, w: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(w, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_formatted() {
        let err = Error {
            user: Some("Bilbo".to_string()),
            op: Some(String::from("server.Put")),
            kind: Kind::Permission,
            message: String::from("uh oh"),
            inner_err: Some(InnerErr(Box::new(StrError(String::from("inner error"))))),
        };

        let formatted = format!("{}", err);
        assert_eq!(
            formatted,
            "{ user: 'Bilbo', op: 'server.Put', kind: 'Permission', err: 'inner error' }",
        );
    }
}
