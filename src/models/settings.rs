#[derive(Debug)]
pub struct Settings {
  pub id: String,
  pub user_id: String,
  pub posts_per_page: i32,
  pub display_email: bool
}

impl Settings {
  pub fn new() -> Self {
    Self {
      id: String::new(),
      user_id: String::new(),
      posts_per_page: 10,
      display_email: false
    }
  }
}
