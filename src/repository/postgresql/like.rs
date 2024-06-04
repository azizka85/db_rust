use std::error;

use postgres;

use crate::repository;

use super::utils;

pub struct Like {}

impl repository::Like for Like {
  fn create(&self, user_id: &str, post_id: &str) -> Result<(), Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.create_wt(user_id, post_id, &mut transaction);

    transaction.commit()?;

    res 
  }

  fn delete(&self, user_id: &str, post_id: &str) -> Result<(), Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.delete_wt(user_id, post_id, &mut transaction);

    transaction.commit()?;

    res
  }
}

impl Like {
  pub fn new() -> Self {
    Self {}
  }

  pub fn create_wt(
    &self, 
    user_id: &str, post_id: &str,
    transaction: &mut postgres::Transaction
  ) -> Result<(), Box<dyn error::Error>> {
    transaction.execute(
      "
        insert into likes(user_id, post_id) 
        values ($1, $2);
      ", 
      &[&user_id.parse::<i32>()?, &post_id.parse::<i32>()?]
    )?;

    Ok(())
  }

  pub fn delete_wt(
    &self, 
    user_id: &str, post_id: &str,
    transaction: &mut postgres::Transaction
  ) -> Result<(), Box<dyn error::Error>> {
    transaction.execute(
      "
        delete from 
          likes 
        where 
          user_id = $1
          and post_id = $2;
      ", 
      &[&user_id.parse::<i32>()?, &post_id.parse::<i32>()?]
    )?;

    Ok(())
  }
}
