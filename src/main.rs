extern crate dotenv;
use dotenv::dotenv;
use std::env;

use actix_web::{App, HttpServer};

mod database;
mod service;
mod routes;
mod models;
use routes::user_routes::{
    created_user,
    login,
    get_user
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let addrs: String = match env::var("ADDRS") {
        Ok(v) => v,
        Err(_) => format!("Error: not impossible loade file env")
    };

    println!("running server...");

    HttpServer::new(|| {
        App::new()
            .service(login)
            .service(created_user)
            .service(get_user)
    })
    .bind((addrs.as_str(), 3000))?
    .run()
    .await
}
