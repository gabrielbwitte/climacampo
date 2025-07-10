extern crate dotenv;
use dotenv::dotenv;
use mongodb::{Client, Collection, Database};
use std::env;

use crate::models::user_model::{Session, User};


async fn db_connection() -> Database {
    dotenv().ok();

    let uri = match env::var("DATABASE") {
        Ok(v) => v,
        Err(_) => panic!("Not impossible loade DATABASE uri.")
    };

    let client = Client::with_uri_str(uri).await.unwrap();
    let db= client.database("climacampoDB");

    db
}

pub async fn user_col() -> Collection<User> {
    let db = db_connection().await;
    let user_col: Collection<User> = db.collection("User");
    user_col
}

pub async fn session() -> Collection<Session> {
    let db = db_connection().await;
    let user_session: Collection<Session> = db.collection("Session");
    user_session
}