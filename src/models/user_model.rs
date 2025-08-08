use mongodb::bson::{oid::ObjectId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct IdDoc {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
}

#[derive(Debug, Clone,Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub email: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub access: Access,
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
    pub start_date: i64,
    pub access: Access
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Access {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub c_producer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modules: Option<Vec<String>>,
}
