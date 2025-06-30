
use std::env;
extern crate dotenv;
use dotenv::dotenv;


use mongodb::{
    Client, Collection
};

use crate::models::model_user::{User, ServiceData};

impl ServiceData {
    pub async fn init() -> Self {
        dotenv().ok();
        let uri = match env::var("DATABASE") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error: não pode ler a variavel de ambiente para conectar ao banco de dados"),
        };
        let client = Client::with_uri_str(uri).await.unwrap();
        let db = client.database("climacampoDB");
        let user_col: Collection<User> = db.collection("User");
        ServiceData { user_col }
    }
}
