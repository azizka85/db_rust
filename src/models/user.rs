use super::Settings;

#[derive(Debug)]
pub struct User {
  pub id: String,
  pub first_name: String,
  pub last_name: String,
  pub email: Option<String>,
  pub password: Option<String>,

  pub settings: Settings  
}

impl User {
  pub fn new() -> Self {
    Self {
      id: String::new(),
      first_name: String::new(),
      last_name: String::new(),
      email: None,
      password: None,

      settings: Settings::new()
    }
  }
}
