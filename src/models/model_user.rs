use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use mongodb::Collection;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub location: String,
    pub title: String,
}

pub struct ServiceData {
    pub user_col: Collection<User>
}