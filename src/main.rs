mod api;
mod models;
mod database;

use actix_web::{web::Data, App, HttpServer};
use api::user_api::{
    create_user,
    get_user,
    update_user,
    delete_user,
    get_all_users,
};
use database::mongodb_data::MongoData;
use std::env;
use dotenv::dotenv;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let addrs: String = match env::var("ADDRS") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error: não foi possivel ler o env ADDRS"),
        };
    let var_port = match env::var("PORT") {
            Ok(v) => v.to_string(),
            Err(_) => format!("Error: não foi possivel ler o env ADDRS"),
        };

    let port: u16 = var_port.parse().expect("Error de tipo da env PORT");
    
    println!("Start climacampo");
    let db = MongoData::init().await;
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
    .bind((addrs.as_str(), port))?
    .run()
    .await
}