use mongodb::bson::{oid::ObjectId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub access: Access,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Access {
    pub c_d_user: bool,
    pub get_users: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub token: String,
    pub start_date: i64
}
