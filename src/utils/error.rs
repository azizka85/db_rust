use std::{fmt, error};

#[derive(Debug)]
pub struct StringError(String);

impl StringError {
  pub fn new(message: &str) -> Self {
    Self(message.to_owned())
  }
}

impl fmt::Display for StringError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl error::Error for StringError { }
