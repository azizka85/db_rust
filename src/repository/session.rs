use std::error;

pub trait Session {
  fn get_user_id(&self, code: &str) -> Result<String, Box<dyn error::Error>>;

  fn create(&self, user_id: &str, code: &str) -> Result<(), Box<dyn error::Error>>;
}
