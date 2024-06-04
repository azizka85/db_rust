use std::error;

use mongodb::{
  bson::{doc, oid::ObjectId, Bson, Document}, 
  options::FindOneOptions, sync::ClientSession
};

use crate::repository;
use crate::models;

use crate::utils::error::StringError;

use super::utils;

pub struct User {}

impl repository::User for User {
  fn create(&self, user: &mut models::User) -> Result<String, Box<dyn error::Error>> {
      let client = utils::connect()?;
      let mut session = client.start_session(None)?;      
      
      session.start_transaction(None)?;

      let res = self.create_ws(user, &mut session);

      session.commit_transaction()?;

      res
  }

  fn get_id(&self, email: &str, password: &str) -> Result<String, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.get_id_ws(email, password, &mut session);

    session.commit_transaction()?;

    res
  }

  fn get_user_settings(&self, id: &str) -> Result<models::User, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.get_user_settings_ws(id, &mut session);

    session.commit_transaction()?;

    res
  }

  fn edit(&self, settings: &models::Settings) -> Result<(), Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.edit_ws(settings, &mut session);

    session.commit_transaction()?;

    res
  }
}

impl User {
  pub fn new() -> Self {
    Self {  }
  }

  pub fn create_ws(
    &self, 
    user: &mut models::User, 
    session: &mut ClientSession
  ) -> Result<String, Box<dyn error::Error>> {
    if let Some(password) = user.password.as_ref() {
      let password = format!("{:x}", md5::compute(password));

      let res = session.client().default_database().unwrap()
        .collection("users")
        .insert_one_with_session(
          doc! {
            "first_name": &user.first_name,
            "last_name": &user.last_name,
            "email": &user.email,
            "password": &password,

            "settings": doc! {
              "posts_per_page": user.settings.posts_per_page,
              "display_email": user.settings.display_email
            },

            "sessions": Vec::<String>::new()
          },
          None,
          session
        )?;

      user.id = res.inserted_id.as_object_id().unwrap().to_string();
      user.settings.user_id = user.id.clone();
        
      Ok(user.id.clone())
    } else {
      Err(
        Box::new(
          StringError::new("Password should be non-empty")
        )
      )
    }
  }

  pub fn get_id_ws(
    &self, 
    email: &str, password: &str,
    session: &mut ClientSession
  ) -> Result<String, Box<dyn error::Error>> {
    let password = format!("{:x}", md5::compute(password));

    let res = session.client().default_database().unwrap()
      .collection::<Document>("users")
      .find_one_with_session(
        doc! {
          "email": email,
          "password": password
        }, 
        FindOneOptions::builder()
          .projection(
            doc! {
              "_id": 1
            }
          )
          .build(), 
        session
      )?;

    res.and_then(
      |doc| doc.get("_id")
                        .and_then(|id| id.as_object_id()
                          .and_then(|id| Some(id.to_string())))
    )
    .ok_or(
      Box::new(
        StringError::new("User with this email and password doesn't exist")
      )
    )
  }

  pub fn get_user_settings_ws(
    &self,
    id: &str,
    session: &mut ClientSession
  ) -> Result<models::User, Box<dyn error::Error>> {
    let res = session.client().default_database().unwrap()
      .collection::<Document>("users")
      .find_one_with_session(
        doc! {
          "_id": ObjectId::parse_str(id).unwrap_or(ObjectId::new()) 
        },
        FindOneOptions::builder()
          .projection(
            doc! {
              "first_name": 1,
              "last_name": 1,
              "email": doc! {
                "$cond": doc! {
                  "if": "$settings.display_email",
                  "then": "$email",
                  "else": "$$REMOVE"
                }
              },
              "settings": 1
            }
          )
          .build(),
        session
      )?;

    res.and_then(|doc| Some(self.read(&doc)))
      .ok_or(
        Box::new(
          StringError::new("User with this id doesn't exist")
        )
      )
  }

  pub fn edit_ws(
    &self, 
    settings: &models::Settings,
    session: &mut ClientSession
  ) -> Result<(), Box<dyn error::Error>> {
    session.client().default_database().unwrap()
      .collection::<Document>("users")
      .update_one_with_session(
        doc! {
          "_id": ObjectId::parse_str(&settings.user_id).unwrap_or(ObjectId::new()) 
        },         
        doc! {
          "$set": doc! {
            "settings": doc! {
              "posts_per_page": settings.posts_per_page,
              "display_email": settings.display_email
            }
          }
        }, 
        None,
        session
      )?;

    Ok(())
  }

  pub fn read(&self, doc: &Document) -> models::User {
    let user_id = doc.get("_id")
      .unwrap_or(&Bson::ObjectId(ObjectId::new()))
      .as_object_id()
      .unwrap_or(ObjectId::new())
      .to_string();

    models::User {
      id: user_id.clone(),
      first_name: doc.get("first_name")
        .unwrap_or(&Bson::String(String::new()))
        .as_str()
        .unwrap_or("")
        .to_owned(),
      last_name: doc.get("last_name")
        .unwrap_or(&Bson::String(String::new()))
        .as_str()
        .unwrap_or("")
        .to_owned(),
      email: doc.get("email")
        .and_then(|email| email.as_str()
                                  .and_then(|email| Some(email.to_owned()))),
      password: None,

      settings: doc.get("settings")
        .and_then(
          |doc| doc.as_document()
                        .and_then(|doc| Some(models::Settings {
                          id: String::new(),
                          user_id: user_id,
                          display_email: doc.get("display_email")
                            .unwrap_or(&Bson::Boolean(false))
                            .as_bool()
                            .unwrap_or(false),
                          posts_per_page: doc.get("posts_per_page")
                            .unwrap_or(&Bson::Int32(0))
                            .as_i32()
                            .unwrap_or(0)
                        }))
        )
        .unwrap_or(models::Settings::new())

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

    let user_repository = repository::mongodb::User::new();
    let session_repository = repository::mongodb::Session::new();

    let connection = utils::connect()?;
    let mut session = connection.start_session(None)?;

    session.start_transaction(None)?;

    user_repository.create_ws(&mut user, &mut session)?;

    assert!(!user.id.is_empty());
    assert!(!user.settings.user_id.is_empty());

    let user_id = user_repository.get_id_ws(
      user.email.as_ref().unwrap(), 
      user.password.as_ref().unwrap(), 
      &mut session
    )?;

    assert_eq!(user_id, user.id);

    let mut user_settings = user_repository.get_user_settings_ws(&user.id, &mut session)?;

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

    user_repository.edit_ws(&user_settings.settings, &mut session)?;

    let user_settings_2 = user_repository.get_user_settings_ws(
      &user.id, 
      &mut session
    )?;

    assert_eq!(user_settings_2.email, user.email);
    assert_eq!(user_settings_2.settings.display_email, user_settings.settings.display_email);
    assert_eq!(user_settings_2.settings.posts_per_page, user_settings.settings.posts_per_page);

    let code = uuid::Uuid::new_v4().to_string();

    session_repository.create_ws(
      &user_id, code.as_str(), 
      &mut session
    )?;    

    let user_id = session_repository.get_user_id_ws(
      code.as_str(), 
      &mut session
    )?;

    assert_eq!(user_id, user.id);    

    session.abort_transaction()?;

    Ok(())
  }
}
