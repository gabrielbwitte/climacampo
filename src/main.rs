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
    get_all_users,
    update_access_user,
    update_profile_user
};

use routes::producer_routes::{
    created_producer,
    get_all_producer,
    get_producer,
    update_producer
};

use routes::farms_routes::{
    created_farm,
    get_farms,
    update_farm
};

use routes::fields_routes::{
    created_field,
    get_fields,
    update_field
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
            .service(get_all_users)
            .service(update_access_user)
            .service(update_profile_user)
            .service(created_producer)
            .service(get_all_producer)
            .service(get_producer)
            .service(update_producer)
            .service(created_farm)
            .service(get_farms)
            .service(update_farm)
            .service(created_field)
            .service(get_fields)
            .service(update_field)
    })
    .bind((addrs.as_str(), 3000))?
    .run()
    .await
}