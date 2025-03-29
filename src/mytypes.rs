use std::{error::Error, fmt::Display, io};

#[derive(Debug)]
pub struct MyError(String);

impl MyError {
    pub fn new(msg: String) -> Self {
        Self(msg)
    }
}

impl Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MyError {}

impl From<String> for MyError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MyError {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<io::Error> for MyError {
    fn from(value: io::Error) -> Self {
        Self(format!("{}", value))
    }
}
