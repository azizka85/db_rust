use std::{error, env};

use mongodb::{sync::Client, options::ClientOptions};

pub fn connect() -> Result<Client, Box<dyn error::Error>> {
  let mut client_options = ClientOptions::parse(
    env::var("MONGODB_CS")?
  )?;

  client_options.default_database = Some(
    env::var("MONGODB_DB")?
  );

  Ok(
    Client::with_options(client_options)?
  )
}
