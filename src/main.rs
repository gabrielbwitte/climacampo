mod routes;
mod models;
mod service;
mod database;

use actix_web::{web::Data, App, HttpServer};
use routes::users::{
    create_user,
    get_user,
    update_user,
    delete_user,
    get_all_users,
};

use std::env;
use dotenv::dotenv;

use models::model_user::ServiceData;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let addrs: String = match env::var("ADDRS") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error: não foi possivel ler o env ADDRS"),
        };
    
    println!("Start climacampo");
    let db = ServiceData::init().await;
    let db_data = Data::new(db);

    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .service(create_user)
            .service(get_user)
            .service(update_user)
            .service(delete_user)
            .service(get_all_users)
    })
    .bind((addrs.as_str(), 3000))?
    .run()
    .await
}