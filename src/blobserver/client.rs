pub struct Client {
    remote: String,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            remote: String::from("localhost:4444"),
        }
    }
}

impl Client {}
