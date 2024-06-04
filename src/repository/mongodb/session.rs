use std::error;

use mongodb::{
  bson::{doc, oid::ObjectId, Document}, 
  options::FindOneOptions, sync::ClientSession
};

use crate::repository;

use crate::utils::error::StringError;

use super::utils;

pub struct Session {}

impl repository::Session for Session {
  fn get_user_id(&self, code: &str) -> Result<String, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.get_user_id_ws(code, &mut session);

    session.commit_transaction()?;

    res
  }

  fn create(&self, user_id: &str, code: &str) -> Result<(), Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.create_ws(user_id, code, &mut session);

    session.commit_transaction()?;

    res
  }
}

impl Session {
  pub fn new() -> Self {
    Self {  }
  }

  pub fn get_user_id_ws(
    &self, 
    code: &str, 
    session: &mut ClientSession
  ) -> Result<String, Box<dyn error::Error>> {
    let res = session.client().default_database().unwrap()
      .collection::<Document>("users")
      .find_one_with_session(
        doc! {
          "sessions": code
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
        StringError::new("User with this session code doesn't exist")
      )
    )
  }

  pub fn create_ws(
    &self,
    user_id: &str, code: &str,
    session: &mut ClientSession
  ) -> Result<(), Box<dyn error::Error>> {
    session.client().default_database().unwrap()
      .collection::<Document>("users")
      .update_one_with_session(
        doc! {
          "_id": ObjectId::parse_str(user_id).unwrap_or(ObjectId::new()) 
        }, 
        doc! {
          "$addToSet": doc! {
            "sessions": code
          }
        }, 
        None, 
        session
      )?;

    Ok(())
  }
}
