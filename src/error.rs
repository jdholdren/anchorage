use std::fmt::{Debug, Display};

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize};

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
///
/// Needs to implement send and sync so the larger type, Error, can also implement those.
#[derive(Debug)]
pub struct InnerErr(pub Box<dyn std::error::Error + Send + Sync>);

impl<'de> serde::Deserialize<'de> for InnerErr {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        let e = Error::from_msg(&s, Kind::Internal);

        Ok(InnerErr(Box::new(e)))
    }
}

/// Error type that we can use everywhere, should provide some documentation
/// on what exactly you're trying to do and where it failed.
#[derive(Debug, Deserialize)]
pub struct Error {
    pub op: Option<String>, // What operation you were trying to do
    pub message: String,
    pub kind: Kind, // What kind of error this is
    pub inner_err: Option<InnerErr>,
}

impl Error {
    // The common constructor of an Error from a generic other error
    //
    // For the 'static constraint:
    // https://github.com/pretzelhammer/rust-blog/blob/master/posts/common-rust-lifetime-misconceptions.md#2-if-t-static-then-t-must-be-valid-for-the-entire-program
    pub fn from_err<E: std::error::Error + Send + Sync + 'static>(
        msg: &str,
        err: E,
        kind: Kind,
    ) -> Self {
        Error {
            kind,
            message: msg.to_owned(),
            inner_err: Some(InnerErr(Box::new(err))),
            op: None,
        }
    }

    // Constructor for making an error from a string
    pub fn from_msg(msg: &str, kind: Kind) -> Self {
        Error {
            kind,
            message: msg.to_owned(),
            inner_err: None,
            op: None,
        }
    }
}

impl Display for Error {
    // Should look like:
    // { user: "blah-blah", op: "server.Put", etc }
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "{{ ")?;

        if let Some(op) = &self.op {
            write!(w, "op: '{}' ", op)?;
        }
        write!(w, "kind: '{}' ", self.kind)?;
        write!(w, "message: '{}' ", self.message)?;

        if let Some(err) = &self.inner_err {
            write!(w, "err: '{:?}' ", err)?;
        }

        write!(w, "}}")
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
        s.serialize_field("message", &self.message)?;
        s.serialize_field("op", &self.op)?;
        s.serialize_field("kind", &self.kind.to_string())?;
        if let Some(err) = &self.inner_err {
            s.serialize_field("err", &format!("{:?}", err))?;
        }
        s.end()
    }
}

// Makes this a valid return type for use with the client
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::from_err("error with reqwest", value, Kind::Internal)
    }
}

// Makes this a type that can be returned from an axum handler.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properly_formatted() {
        let err = Error {
            op: Some(String::from("server.Put")),
            kind: Kind::Permission,
            message: String::from("uh oh"),
            inner_err: None,
        };

        let formatted = format!("{}", err);
        assert_eq!(
            formatted,
            "{ op: 'server.Put' kind: 'Permission' message: 'uh oh' }",
        );
    }
}
