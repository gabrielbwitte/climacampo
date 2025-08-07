extern crate dotenv;
use actix_web::http::StatusCode;
use dotenv::dotenv;
use mongodb::options::IndexOptions;
use mongodb::{Client, Collection, Database, IndexModel};
use mongodb::bson::doc;
use std::env;

use crate::models::config_fields_model::Producer;
use crate::models::user_model::{Session, User};


async fn db_connection() -> Database {
    dotenv().ok();

    let uri = match env::var("DATABASE") {
        Ok(v) => v,
        Err(_) => panic!("Not impossible loade DATABASE uri.")
    };

    let client = Client::with_uri_str(uri).await.unwrap();
    client.database("climacampoDB")
}

pub async fn user_col() -> Result<Collection<User>, StatusCode> {
    let db = db_connection().await;
    let user_col: Collection<User> = db.collection("User");

    let options = IndexOptions::builder().unique(true).build();

    let indexes = vec![
        IndexModel::builder()
        .keys(doc! {"username": 1})
        .options(options.clone())
        .build(),
        IndexModel::builder()
        .keys(doc! {"email": 1})
        .options(options)
        .build(),
    ];
    
    match user_col.create_indexes(indexes).await {
        Ok(_) => Ok(user_col),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    } 
    
}

pub async fn session_col() -> Collection<Session> {
    let db = db_connection().await;
    let user_session: Collection<Session> = db.collection("Session");
    user_session
}

pub async fn producer_col() -> Collection<Producer> {
    let db = db_connection().await;
    let producer: Collection<Producer> = db.collection("Producer");
    producer
}