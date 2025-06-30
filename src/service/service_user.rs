
use futures::TryStreamExt;
use mongodb::{
    bson::{extjson::de::Error, oid::ObjectId, doc},
    results::{InsertOneResult, UpdateResult, DeleteResult},   
};
use crate::{models::model_user::{ServiceData, User}};

impl ServiceData {
    pub async fn create_user(&self, new_user: User) -> Result<InsertOneResult, Error> {
        let new_doc = User {
            id: new_user.id,
            name: new_user.name,
            email: new_user.email,
            password: new_user.password,
            level_access: new_user.level_access
        };
        let user = self
            .user_col
            .insert_one(new_doc, None)
            .await
            .ok()
            .expect("Error: Usuario não encontrado");
        
        Ok(user)
    }

    pub async fn get_user(&self, id: &String) -> Result<User, Error> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! {"_id": obj_id};
        
        let user_detail = self
            .user_col
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
                    "email": new_user.email,
                    "password": new_user.password,
                    "level_access": new_user.level_access
                },
        };
        let update_doc = self
            .user_col
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
            .user_col
            .delete_one(filter, None)
            .await
            .ok()
            .expect("Error deleting user");
        Ok(user_detail)
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, Error> {
        let mut cursors = self
            .user_col
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