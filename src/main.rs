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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    .bind(("localhost", 8080))?
    .run()
    .await
}