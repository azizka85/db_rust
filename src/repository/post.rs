use std::error;

use crate::models;

pub trait Post {
  fn create(&self, post: &models::Post) -> Result<String, Box<dyn error::Error>>;

  fn get(&self, id: &str, user_id: Option<&str>) -> Result<models::Post, Box<dyn error::Error>>;

  fn list(&self, user_id: Option<&str>) -> Result<Vec<models::Post>, Box<dyn error::Error>>;

  fn liked_list(&self, user_id: &str) -> Result<Vec<models::Post>, Box<dyn error::Error>>;
}
