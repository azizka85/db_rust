use super::User;

#[derive(Debug)]
pub struct Post {
  pub id: String,
  pub title: String,
  pub text: Option<String>,
  pub description: Option<String>,
  pub liked: bool,
  
  pub author: Option<User>
}
