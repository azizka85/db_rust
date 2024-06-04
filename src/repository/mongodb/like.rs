use std::error;

use mongodb::{
  bson::{doc, oid::ObjectId, Document}, 
  sync::ClientSession
};

use crate::repository;

use super::utils;

pub struct Like {}

impl repository::Like for Like {
  fn create(&self, user_id: &str, post_id: &str) -> Result<(), Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.create_ws(user_id, post_id, &mut session);

    session.commit_transaction()?;

    res
  }

  fn delete(&self, user_id: &str, post_id: &str) -> Result<(), Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.delete_ws(user_id, post_id, &mut session);

    session.commit_transaction()?;

    res
  }
}

impl Like {
  pub fn new() -> Self {
    Self {  }
  }

  pub fn create_ws(
    &self, 
    user_id: &str, post_id: &str,
    session: &mut ClientSession
  ) -> Result<(), Box<dyn error::Error>> {
    session.client().default_database().unwrap()
      .collection("likes")
      .insert_one_with_session(
        doc! {
          "user_id": ObjectId::parse_str(user_id)?,
          "post_id": ObjectId::parse_str(post_id)?
        },
        None,
        session
      )?;

    Ok(())
  }

  pub fn delete_ws(
    &self, 
    user_id: &str, post_id: &str,
    session: &mut ClientSession
  ) -> Result<(), Box<dyn error::Error>> {
    session.client().default_database().unwrap()
      .collection::<Document>("likes")
      .delete_one_with_session(
        doc! {
          "user_id": ObjectId::parse_str(user_id)?,
          "post_id": ObjectId::parse_str(post_id)?
        }, 
        None, 
        session
      )?;

    Ok(())
  }
}
