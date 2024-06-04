use std::error;

use crate::models;

pub trait User {
  fn create(&self, user: &mut models::User) -> Result<String, Box<dyn error::Error>>;

  fn get_id(&self, email: &str, password: &str) -> Result<String, Box<dyn error::Error>>;

  fn get_user_settings(&self, id: &str) -> Result<models::User, Box<dyn error::Error>>;

  fn edit(&self, settings: &models::Settings) -> Result<(), Box<dyn error::Error>>;
}
