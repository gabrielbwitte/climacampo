use std::env;
extern crate dotenv;
use dotenv::dotenv;

use futures::TryStreamExt;
use mongodb::{
    bson::{extjson::de::Error, oid::ObjectId, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},
    Client,
    Collection,
    
};

use crate::models::model_user::User;

pub struct MongoData {
    col: Collection<User>
}

impl MongoData {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("DATABASE") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error: não pode ler a variavel de ambiente para conectar ao banco de dados"),
        };
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("climacampoDB");
        let col: Collection<User> = db.collection("User");
        MongoData { col }
    }

    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: new_user.id,
            name: new_user.name,
            location: new_user.location,
            title: new_user.title,
        };
        let user = self
            .col
            .insert_one(new_doc, None)
            .await
            .ok()
            .expect("Error: Usuario não criado");
        Ok(user)
    }

    pub async fn get_user(&self, id: &String) -> Result<User, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let user_detail = self
            .col
            .find_one(filter, None)
            .await
            .ok()
            .expect("Error getting user's detail");
        Ok(user_detail.unwrap())
    }

    pub async fn update_user(&self, id: &String, new_user: User) -> Result<UpdateResult, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let new_doc = doc! {
            "$set": 
                {
                    "id": new_user.id,
                    "name": new_user.name,
                    "location": new_user.location,
                    "title": new_user.title
                },
        };
        let update_doc = self
            .col
            .update_one(filter, new_doc, None)
            .await
            .ok()
            .expect("Error updating user");
        Ok(update_doc)
    }

    pub async fn delete_user(&self, id: &String) -> Result<DeleteResult, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        let user_detail = self
            .col
            .delete_one(filter, None)
            .await
            .ok()
            .expect("Error deleting user");
        Ok(user_detail)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, Error> {
        let mut cursors = self
            .col
            .find(None, None)
            .await
            .ok()
            .expect("Error getting list of users");
        let mut users: Vec<User> = Vec::new();
        while let Some(user) = cursors 
            .try_next()
            .await
            .ok()
            .expect("Error mapping though cursor")
        {
            users.push(user);
        }
        Ok(users)
    }

}