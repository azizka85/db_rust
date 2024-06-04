use std::error;

use postgres;

use crate::repository;

use super::utils;

pub struct Session {}

impl repository::Session for Session {
  fn get_user_id(&self, code: &str) -> Result<String, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.get_user_id_wt(code, &mut transaction);

    transaction.commit()?;

    res
  }

  fn create(&self, user_id: &str, code: &str) -> Result<(), Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.create_wt(user_id, code, &mut transaction);

    transaction.commit()?;

    res
  }
}

impl Session {
  pub fn new() -> Self {
    Self { }
  }

  pub fn get_user_id_wt(
    &self, 
    code: &str, 
    transaction: &mut postgres::Transaction
  ) -> Result<String, Box<dyn error::Error>> {
    let row = transaction.query_one(
      "select user_id from sessions where code = $1;", 
      &[&code]
    )?;

    let user_id: i32 = row.get("user_id");

    Ok(user_id.to_string())
  }

  pub fn create_wt(
    &self,
    user_id: &str, code: &str,
    transaction: &mut postgres::Transaction
  ) -> Result<(), Box<dyn error::Error>> {
    transaction.execute(
      "
        insert into sessions(user_id, code) 
        values ($1, $2);
      ", 
      &[&user_id.parse::<i32>()?, &code]
    )?;

    Ok(())
  }
}
