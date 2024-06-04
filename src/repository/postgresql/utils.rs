use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

use postgres::{Client, Config};
use postgres_openssl::MakeTlsConnector;

use std::{env, error};

pub fn connect() -> Result<Client, Box<dyn error::Error>> {
  let mut builder = SslConnector::builder(SslMethod::tls())?;

  builder.set_verify(SslVerifyMode::NONE);

  let connector = MakeTlsConnector::new(builder.build());

  let mut config = Config::new();

  config.host(&env::var("POSTGRES_HOST")?);
  config.user(&env::var("POSTGRES_USER")?);
  config.password(&env::var("POSTGRES_PASSWORD")?);
  config.dbname(&env::var("POSTGRES_DB")?);

  let client = config.connect(connector)?;

  Ok(client)
}
