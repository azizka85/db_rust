use std::error;

use postgres;
use postgres::fallible_iterator::FallibleIterator;

use crate::repository;
use crate::models;

use super::utils;

pub struct Post {}

impl repository::Post for Post {
  fn create(&self, post: &models::Post) -> Result<String, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.create_wt(post, &mut transaction);

    transaction.commit()?;

    res
  }

  fn get(&self, id: &str, user_id: Option<&str>) -> Result<models::Post, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.get_wt(id, user_id, &mut transaction);

    transaction.commit()?;

    res
  }

  fn list(&self, user_id: Option<&str>) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.list_wt(user_id, &mut transaction);

    transaction.commit()?;

    res
  }

  fn liked_list(&self, user_id: &str) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let mut connection = utils::connect()?;

    let mut transaction = connection.transaction()?;

    let res = self.liked_list_wt(user_id, &mut transaction);

    transaction.commit()?;

    res
  }
}

impl Post {
  pub fn new() -> Self {
    Self { }
  }

  pub fn create_wt(
    &self, 
    post: &models::Post,
    transaction: &mut postgres::Transaction
  ) -> Result<String, Box<dyn error::Error>> {
    let mut user_id = None;

    if let Some(author) = post.author.as_ref() {
      user_id = Some(author.id.parse::<i32>()?);
    }

    let row = transaction.query_one(
      "
        insert into posts(user_id, title, text, abstract)
        values ($1, $2, $3, $4)
        returning id;
      ", 
      &[&user_id, &post.title, &post.text, &post.description]
    )?;

    let post_id: i32 = row.get(0);
    
    Ok(post_id.to_string())
  }

  pub fn get_wt(
    &self, 
    id: &str, user_id: Option<&str>,
    transaction: &mut postgres::Transaction
  ) -> Result<models::Post, Box<dyn error::Error>> {
    let user_id = user_id.and_then(|s| s.parse::<i32>().ok());

    let row = transaction.query_one(
      "
        select 
          u.id user_id, u.first_name, u.last_name,
          case
            when s.display_email is null or s.display_email = false then null
            else u.email
          end email,
          p.id post_id, p.title, p.text, p.abstract,
          case
            when l.id is null then false
            else true
          end liked
        from 
          posts p
        left join
          users u 
          on p.user_id = u.id
        left join 
          settings s
          on p.user_id = s.user_id
        left join
          likes l
          on p.id = l.post_id
            and l.user_id = $2
        where
          p.id = $1;
      ",
      &[&id.parse::<i32>()?, &user_id]
    )?;

    Ok(self.read(&row))
  }

  pub fn list_wt(
    &self, 
    user_id: Option<&str>,
    transaction: &mut postgres::Transaction
  ) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let user_id = user_id.and_then(|s| s.parse::<i32>().ok());

    let mut v = Vec::new();

    let mut row_iter = transaction.query_raw(
      "
        select 
          u.id user_id, u.first_name, u.last_name,
          case
            when s.display_email is null or s.display_email = false then null
            else u.email
          end email,
          p.id post_id, p.title, p.text, p.abstract,
          case
            when l.id is null then false
            else true
          end liked
        from 
          posts p
        left join
          users u 
          on p.user_id = u.id
        left join 
          settings s
          on p.user_id = s.user_id
        left join
          likes l
          on p.id = l.post_id
            and l.user_id = $1;
      ", 
      &[&user_id]
    )?;

    while let Some(row) = row_iter.next()? {
      v.push(
        self.read(&row)
      );
    }

    Ok(v)
  }

  pub fn liked_list_wt(
    &self, 
    user_id: &str,
    transaction: &mut postgres::Transaction
  ) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let mut v = Vec::new();

    let mut row_iter = transaction.query_raw(
      "
        select 
          u.id user_id, u.first_name, u.last_name,
          case
            when s.display_email is null or s.display_email = false then null
            else u.email
          end email,
          p.id post_id, p.title, p.text, p.abstract,
          case
            when l.id is null then false
            else true
          end liked
        from 
          posts p
        left join
          users u 
          on p.user_id = u.id
        left join 
          settings s
          on p.user_id = s.user_id
        inner join
          likes l
          on p.id = l.post_id
            and l.user_id = $1;
      ",
      &[&user_id.parse::<i32>()?]
    )?;

    while let Some(row) = row_iter.next()? {
      v.push(
        self.read(&row)
      );
    }

    Ok(v)
  }

  pub fn read(&self, row: &postgres::Row) -> models::Post {
    let user_id: Option<i32> = row.get("user_id");
    let post_id: i32 = row.get("post_id");
    
    let post_id = post_id.to_string();

    models::Post {
      id: post_id,
      title: row.get("title"),
      text: row.get("text"),
      description: row.get("abstract"),
      liked: row.get("liked"),

      author: if user_id.is_some() {
        Some(
          models::User {
            id: user_id.unwrap().to_string(),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),
            email: row.get("email"),
            password: None,
            
            settings: models::Settings::new()
          }
        )
      } else {
        None
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
  fn test_post() -> Result<(), Box<dyn error::Error>> {
    dotenv().ok();

    let mut user = models::User {
      id: String::new(),
      first_name: "__test_2__".to_owned(),
      last_name: "__test_2__".to_owned(),
      email: Some("__test_2__@2.again".to_owned()),
      password: Some("test".to_owned()),

      settings: models::Settings::new()
    };    

    let user_repository = repository::postgresql::User::new();
    let post_repository = repository::postgresql::Post::new();
    let like_repository = repository::postgresql::Like::new();

    let mut connection = utils::connect()?;
    let mut transaction = connection.transaction()?;

    user_repository.create_wt(&mut user, &mut transaction)?;

    assert!(!user.id.is_empty());

    let post = models::Post {
      id: String::new(),
      title: "__title_1".to_owned(),
      text: Some("__text_1".to_owned()),
      description: Some("__abstract_1".to_owned()),
      liked: false,
      author: Some(user)
    };

    let user = post.author.as_ref().unwrap();

    let post_id = post_repository.create_wt(&post, &mut transaction)?;

    assert!(!post_id.is_empty());

    let post_by_id = post_repository.get_wt(
      &post_id, Some(&user.id), 
      &mut transaction
    )?;

    assert_eq!(post_by_id.id, post_id);
    assert_eq!(post_by_id.title, post.title);
    assert_eq!(post_by_id.text, post.text);
    assert_eq!(post_by_id.description, post.description);
    assert_eq!(post_by_id.liked, post.liked);

    assert_eq!(post_by_id.author.as_ref().unwrap().id, user.id);
    assert_eq!(post_by_id.author.as_ref().unwrap().first_name, user.first_name);
    assert_eq!(post_by_id.author.as_ref().unwrap().last_name, user.last_name);

    if user.settings.display_email {
      assert_eq!(post_by_id.author.as_ref().unwrap().email, user.email);
    } else {
      assert!(post_by_id.author.as_ref().unwrap().email.is_none());
    }

    let posts = post_repository.list_wt(Some(&user.id), &mut transaction)?;

    assert!(posts.len() > 0);
    assert!(posts.iter().find(|p| p.id == post_id).is_some());

    let liked_posts = post_repository.liked_list_wt(&user.id, &mut transaction)?;

    assert_eq!(liked_posts.len(), 0);

    like_repository.create_wt(&user.id, &post_id, &mut transaction)?;

    let liked_posts_2 = post_repository.liked_list_wt(&user.id, &mut transaction)?;

    assert!(liked_posts_2.len() > 0);
    assert!(liked_posts_2.iter().find(|p| p.id == post_id && p.liked).is_some());

    like_repository.delete_wt(&user.id, &post_id, &mut transaction)?;

    let liked_posts_3 = post_repository.liked_list_wt(
      &user.id, 
      &mut transaction
    )?;

    assert_eq!(liked_posts_3.len(), 0);

    transaction.rollback()?;

    Ok(())
  }
}
