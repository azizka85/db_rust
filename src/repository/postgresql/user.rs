use std::error;

use postgres;

use crate::repository;
use crate::models;

use crate::utils::error::StringError;

use super::utils;

pub struct User {}

impl repository::User for User {
  fn create(&self, user: &mut models::User) -> Result<String, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.create_wt(user, &mut transaction);

    transaction.commit()?;

    res
  }

  fn get_id(&self, email: &str, password: &str) -> Result<String, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.get_id_wt(email, password, &mut transaction);

    transaction.commit()?;

    res
  }

  fn get_user_settings(&self, id: &str) -> Result<models::User, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.get_user_settings_wt(id, &mut transaction);

    transaction.commit()?;

    res
  }

  fn edit(&self, settings: &models::Settings) -> Result<(), Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.edit_wt(settings, &mut transaction);

    transaction.commit()?;

    res
  }
}

impl User {
  pub fn new() -> Self {
    Self { }
  }

  pub fn create_wt(
    &self, 
    user: &mut models::User, 
    transaction: &mut postgres::Transaction
  ) -> Result<String, Box<dyn error::Error>> {
    let user_id = self.create_user_wt(user, transaction)?;

    user.id = user_id.clone();
    user.settings.user_id = user_id.clone();

    let settings_id = self.create_settings_wt(&user.settings, transaction)?;

    user.settings.id = settings_id;
    
    Ok(user_id)
  }

  pub fn create_user_wt(
    &self, 
    user: &models::User, 
    transaction: &mut postgres::Transaction
  ) -> Result<String, Box<dyn error::Error>> {
    if let Some(password) = user.password.as_ref() {
      let password = format!("{:x}", md5::compute(password));

      let row = transaction.query_one(
        "
          insert into users(first_name, last_name, email, password) 
          values ($1, $2, $3, $4)
          returning id;
        ",
        &[&user.first_name, &user.last_name, &user.email, &password]
      )?;

      let user_id: i32 = row.get(0);      

      Ok(user_id.to_string())
    } else {
      Err(
        Box::new(
          StringError::new("Password should be non-empty")
        )
      )
    }
  }

  pub fn create_settings_wt(
    &self, 
    settings: &models::Settings, 
    transaction: &mut postgres::Transaction
  ) -> Result<String, Box<dyn error::Error>> {
    let row = transaction.query_one(
      "
        insert into settings(user_id, posts_per_page, display_email) 
        values ($1, $2, $3)
        returning id;
      ",
      &[&settings.user_id.parse::<i32>()?, &settings.posts_per_page, &settings.display_email]
    )?;

    let settings_id: i32 = row.get(0);

    Ok(settings_id.to_string())
  }

  pub fn get_id_wt(
    &self, 
    email: &str, password: &str,
    transaction: &mut postgres::Transaction
  ) -> Result<String, Box<dyn error::Error>> {
    let password = format!("{:x}", md5::compute(password));

    let row = transaction.query_one(
      "select id from users where email = $1 and password = $2;", 
      &[&email, &password]
    )?;

    let user_id: i32 = row.get("id");

    Ok(user_id.to_string())
  }

  pub fn get_user_settings_wt(
    &self,
    id: &str,
    transaction: &mut postgres::Transaction
  ) -> Result<models::User, Box<dyn error::Error>> {
    let row = transaction.query_one(
      "
        select 
          u.id user_id, u.first_name, u.last_name, 
          case
            when s.display_email = false then null
            else u.email
          end email, 
          s.id settings_id, s.posts_per_page, s.display_email
        from
          users u, settings s
        where
          s.user_id = u.id
          and u.id = $1;
      ", 
      &[&id.parse::<i32>()?]
    )?;

    Ok(self.read(&row))
  }

  pub fn edit_wt(
    &self, 
    settings: &models::Settings, 
    transaction: &mut postgres::Transaction
  ) -> Result<(), Box<dyn error::Error>> {
    transaction.execute(
      "
        update 
          settings 
        set
          posts_per_page = $1,
          display_email = $2
        where
          user_id = $3;
      ", 
      &[&settings.posts_per_page, &settings.display_email, &settings.user_id.parse::<i32>()?]
    )?;

    Ok(())
  }

  pub fn read(&self, row: &postgres::Row) -> models::User {
    let user_id: i32 = row.get("user_id");
    let settings_id: i32 = row.get("settings_id");

    let user_id = user_id.to_string();
    let settings_id = settings_id.to_string();

    models::User {
      id: user_id.clone(),
      first_name: row.get("first_name"),
      last_name: row.get("last_name"),
      email: row.get("email"),
      password: None,

      settings: models::Settings {
        id: settings_id,
        user_id: user_id,
        display_email: row.get("display_email"),
        posts_per_page: row.get("posts_per_page")
      }
    }
  }  
}

#[cfg(test)]
mod tests {
  use std::error;

  use dotenv::dotenv;

  use super::utils;
  use crate::{models, repository};

  #[test]
  fn test_user() -> Result<(), Box<dyn error::Error>> {
    dotenv().ok();

    let mut user = models::User {
      id: String::new(),
      first_name: "__test_1__".to_owned(),
      last_name: "__test_1__".to_owned(),
      email: Some("__test_1__@1.again".to_owned()),
      password: Some("test".to_owned()),

      settings: models::Settings::new()
    };

    let user_repository = repository::postgresql::User::new();
    let session_repository = repository::postgresql::Session::new();

    let mut connection = utils::connect()?;
    let mut transaction = connection.transaction()?;

    user_repository.create_wt(&mut user, &mut transaction)?;

    assert!(!user.id.is_empty());
    assert!(!user.settings.id.is_empty());

    let user_id = user_repository.get_id_wt(
      user.email.as_ref().unwrap(), 
      user.password.as_ref().unwrap(), 
      &mut transaction
    )?;

    assert_eq!(user_id, user.id);

    let mut user_settings = user_repository.get_user_settings_wt(&user.id, &mut transaction)?;

    assert_eq!(user_settings.id, user.id);
    assert_eq!(user_settings.first_name, user.first_name);
    assert_eq!(user_settings.last_name, user.last_name);

    if user.settings.display_email {
      assert_eq!(user_settings.email, user.email);
    } else {
      assert!(user_settings.email.is_none());
    }

    assert_eq!(user_settings.settings.id, user.settings.id);
    assert_eq!(user_settings.settings.user_id, user.id);
    assert_eq!(user_settings.settings.display_email, user.settings.display_email);
    assert_eq!(user_settings.settings.posts_per_page, user.settings.posts_per_page);

    user_settings.settings.posts_per_page = 30;
    user_settings.settings.display_email = true;

    user_repository.edit_wt(&user_settings.settings, &mut transaction)?;

    let user_settings_2 = user_repository.get_user_settings_wt(
      &user.id, 
      &mut transaction
    )?;

    assert_eq!(user_settings_2.email, user.email);
    assert_eq!(user_settings_2.settings.display_email, user_settings.settings.display_email);
    assert_eq!(user_settings_2.settings.posts_per_page, user_settings.settings.posts_per_page);

    let code = uuid::Uuid::new_v4().to_string();

    session_repository.create_wt(
      &user_id, code.as_str(), 
      &mut transaction
    )?;    

    let user_id = session_repository.get_user_id_wt(
      code.as_str(), 
      &mut transaction
    )?;

    assert_eq!(user_id, user.id);    

    transaction.rollback()?;

    Ok(())
  }
}
