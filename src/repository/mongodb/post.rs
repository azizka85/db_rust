use std::error;

use mongodb::{bson::{doc, oid::ObjectId, Bson, Document}, sync::ClientSession};

use crate::repository;
use crate::models;

use crate::utils::error::StringError;

use super::utils;

pub struct Post {}

impl repository::Post for Post {
  fn create(&self, post: &models::Post) -> Result<String, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.create_ws(post, &mut session);

    session.commit_transaction()?;

    res
  }

  fn get(&self, id: &str, user_id: Option<&str>) -> Result<models::Post, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.get_ws(id, user_id, &mut session);

    session.commit_transaction()?;

    res
  }

  fn list(&self, user_id: Option<&str>) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.list_ws(user_id, &mut session);

    session.commit_transaction()?;

    res
  }

  fn liked_list(&self, user_id: &str) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let client = utils::connect()?;
    let mut session = client.start_session(None)?;      
    
    session.start_transaction(None)?;

    let res = self.liked_list_ws(user_id, &mut session);

    session.commit_transaction()?;

    res
  }
}

impl Post {
  pub fn new() -> Self {
    Self {  }
  }

  pub fn create_ws(
    &self, 
    post: &models::Post,
    session: &mut ClientSession
  ) -> Result<String, Box<dyn error::Error>> {
    let res = session.client().default_database().unwrap()
      .collection("posts")
      .insert_one_with_session(
        doc! {
          "user_id": post.author.as_ref().and_then(|user| 
            ObjectId::parse_str(&user.id).ok()
          ),
          "title": &post.title,
          "text": &post.text,
          "description": &post.description
        }, 
        None, 
        session
      )?;

    Ok(res.inserted_id.as_object_id().unwrap().to_string())
  }

  pub fn get_ws(
    &self, 
    id: &str, user_id: Option<&str>,
    session: &mut ClientSession
  ) -> Result<models::Post, Box<dyn error::Error>> {
    let mut pipeline = vec![
      doc! {
        "$match": doc! {
          "_id": ObjectId::parse_str(id)?
        }
      }
    ];

    pipeline.extend(self.pipeline(&user_id));

    let mut cursor = session.client().default_database().unwrap()
      .collection::<Document>("posts")
      .aggregate_with_session(
        pipeline, 
        None,
        session  
      )?;

    let mut post = None;

    while let Some(doc) = cursor.next(session) {
      post = Some(self.read(&doc?))
    }

    if post.is_some() {
      Ok(post.unwrap())
    } else {
      Err(
        Box::new(
          StringError::new(
            "Post with this id not found"
          )
        )
      )
    }
  }

  pub fn list_ws(
    &self, 
    user_id: Option<&str>,
    session: &mut ClientSession
  ) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let mut data = Vec::new();

    let mut cursor = session.client().default_database().unwrap()
      .collection::<Document>("posts")
      .aggregate_with_session(
        self.pipeline(&user_id), 
        None,
        session  
      )?;

    while let Some(doc) = cursor.next(session) {
      data.push(
        self.read(&doc?)
      );
    }

    Ok(data)
  }

  pub fn liked_list_ws(
    &self, 
    user_id: &str,
    session: &mut ClientSession
  ) -> Result<Vec<models::Post>, Box<dyn error::Error>> {
    let mut data = Vec::new();

    let mut pipeline = self.pipeline(&Some(user_id));

    pipeline.push(
      doc! {
        "$match": doc! {
          "liked": true
        }
      }
    );

    let mut cursor = session.client().default_database().unwrap()
      .collection::<Document>("posts")
      .aggregate_with_session(
        pipeline, 
        None,
        session  
      )?;

    while let Some(doc) = cursor.next(session) {
      data.push(
        self.read(&doc?)
      );
    }

    Ok(data)
  }

  pub fn read(&self, doc: &Document) -> models::Post {
    models::Post {
      id: doc.get("_id")
        .unwrap_or(&Bson::ObjectId(ObjectId::new()))
        .as_object_id()
        .unwrap_or(ObjectId::new())
        .to_string(),
      title: doc.get("title")
        .unwrap_or(&Bson::String(String::new()))
        .as_str()
        .unwrap_or("")
        .to_owned(),
      text: doc.get("text")
        .and_then(|text| text.as_str()
          .and_then(|text| Some(text.to_owned()))),
      description: doc.get("description")
        .and_then(|description: &Bson| description.as_str()
          .and_then(|description| Some(description.to_owned()))),
      author: doc.get("author")
        .and_then(
          |doc| doc.as_document()
                        .and_then(|doc| Some(models::User {
                          id: doc.get("_id")
                            .unwrap_or(&Bson::ObjectId(ObjectId::new()))
                            .as_object_id()
                            .unwrap_or(ObjectId::new())
                            .to_string(),
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
                          email: None,
                          password: None,
                          
                          settings: models::Settings::new()
                        }))
        ),
      liked: doc.get("liked")
        .and_then(|doc| doc.as_bool())
        .unwrap_or(false)        
    }
  }

  pub fn pipeline(&self, user_id: &Option<&str>) -> Vec<Document> {
    vec![
      doc! {
        "$lookup": doc! {
          "from": "users",
          "localField": "user_id",
          "foreignField": "_id",
          "as": "author"
        }
      },
      doc! {
        "$set": doc! {
          "author": doc! {
            "$first": "$author"
          }
        }
      },
      doc! {
        "$unset": vec![
          "user_id",
          "author.sessions",
          "author.settings",
          "author.email",
          "author.password"              
        ]
      },
      doc! {
        "$lookup": doc! {
          "from": "likes",
          "let": doc! {
            "post_id": "$_id"
          },
          "pipeline": vec![
            doc! {
              "$match": doc! {
                "$expr": doc! {
                  "$and": vec![
                    doc! {"$eq": vec!["$post_id", "$$post_id"]},
                    doc! {"$eq": vec![
                      Bson::String("$user_id".to_owned()), 
                      Bson::ObjectId(
                        user_id.and_then(|user_id| ObjectId::parse_str(user_id).ok())
                          .unwrap_or(ObjectId::new())
                      )
                    ]}
                  ]
                }
              }
            }
          ],
          "as": "like"
        }
      },
      doc! {
        "$set": doc! {
          "liked": doc! {
            "$cond": doc! {
              "if": doc! {"$eq": vec![
                Bson::Document(
                  doc! {"$size": "$like"}
                ),
                Bson::Int32(0)
              ]},
              "then": false,
              "else": true
            }
          }
        }
      },
      doc! {
        "$unset": vec!["like"]
      }
    ]
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

    let user_repository = repository::mongodb::User::new();
    let post_repository = repository::mongodb::Post::new();
    let like_repository = repository::mongodb::Like::new();

    let connection = utils::connect()?;
    let mut session = connection.start_session(None)?;

    session.start_transaction(None)?;

    user_repository.create_ws(&mut user, &mut session)?;

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

    let post_id = post_repository.create_ws(&post, &mut session)?;

    assert!(!post_id.is_empty());

    let post_by_id = post_repository.get_ws(
      &post_id, Some(&user.id), 
      &mut session
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

    let posts = post_repository.list_ws(Some(&user.id), &mut session)?;

    assert!(posts.len() > 0);
    assert!(posts.iter().find(|p| p.id == post_id).is_some());

    let liked_posts = post_repository.liked_list_ws(&user.id, &mut session)?;

    assert_eq!(liked_posts.len(), 0);

    like_repository.create_ws(&user.id, &post_id, &mut session)?;

    let liked_posts_2 = post_repository.liked_list_ws(&user.id, &mut session)?;

    assert!(liked_posts_2.len() > 0);
    assert!(liked_posts_2.iter().find(|p| p.id == post_id && p.liked).is_some());

    like_repository.delete_ws(&user.id, &post_id, &mut session)?;

    let liked_posts_3 = post_repository.liked_list_ws(
      &user.id, 
      &mut session
    )?;

    assert_eq!(liked_posts_3.len(), 0);

    session.abort_transaction()?;

    Ok(())
  }
}
