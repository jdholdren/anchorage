#[derive(Debug)]
pub enum Kind {
    Permission,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "{:?}", self)
    }
}

// Error type that we can use everywhere, should provide some documentation
// on what exactly you're trying to do and where it failed
#[derive(Debug)]
pub struct Error<'op> {
    pub user: String,                    // What user was trying to operate
    pub op: &'op str,                    // What operation you were trying to do
    pub kind: Kind,                      // What kind of error this is
    pub err: Box<dyn std::error::Error>, // The inner error
}

impl std::fmt::Display for Error<'_> {
    // Should look like:
    // { user: "blah-blah", op: "server.Put", etc }
    fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(w, "{{ ")?;

        write!(w, "user: '{}', ", self.user)?;
        write!(w, "op: '{}', ", self.op)?;
        write!(w, "kind: '{}', ", self.kind)?;
        write!(w, "err: '{}'", self.err)?;

        write!(w, " }}")
    }
}

impl std::error::Error for Error<'_> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct InnerError {}

    impl std::fmt::Display for InnerError {
        fn fmt(&self, w: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(w, "some error")
        }
    }

    impl std::error::Error for InnerError {}

    #[test]
    fn properly_formatted() {
        let err: Box<dyn std::error::Error> = Box::new(Error {
            user: "Bilbo".to_string(),
            op: "server.Put",
            kind: Kind::Permission,
            err: Box::new(InnerError {}),
        });

        let formatted = format!("{}", err);
        assert_eq!(
            formatted,
            "{ user: 'Bilbo', op: 'server.Put', kind: 'Permission', err: 'some error' }",
        );
    }
}
