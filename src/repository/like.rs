use std::error;

pub trait Like {
  fn create(&self, user_id: &str, post_id: &str) -> Result<(), Box<dyn error::Error>>;

  fn delete(&self, user_id: &str, post_id: &str) -> Result<(), Box<dyn error::Error>>;
}
