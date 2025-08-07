extern crate dotenv;
use dotenv::dotenv;
use std::env;

use actix_web::{App, HttpServer};

mod database;
mod service;
mod routes;
mod models;
use routes::user_routes::{
    login,
    logoff,
    get_user_profile,
    created_user,
    get_user,
    update_access_user,
    update_profile_user
};

use routes::config_fields_routes::{
    created_producer,
    get_producer,
    update_producer
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let addrs: String = match env::var("ADDRS") {
        Ok(v) => v,
        Err(_) => "Error: not impossible loade file env".to_string()
    };

    println!("running server...");

    HttpServer::new(|| {
        App::new()
            .service(login)
            .service(logoff)
            .service(get_user_profile)
            .service(created_user)
            .service(get_user)
            .service(update_access_user)
            .service(update_profile_user)
            .service(created_producer)
            .service(get_producer)
            .service(update_producer)
    })
    .bind((addrs.as_str(), 3000))?
    .run()
    .await
}