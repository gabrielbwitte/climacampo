use actix_web::{cookie::Cookie, get, patch, post, web::{self, Json}, HttpRequest, HttpResponse};

use crate::{database::mongo_db::{farm_col}, models::property_model::Farms, service::session::authorization};

use mongodb::bson::{doc, oid::ObjectId};
use futures::TryStreamExt;


#[post("/farm")]
pub async fn created_farm(hed: HttpRequest, req: Json<Farms>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<Farms>) -> bool {
        data.name.is_empty()
    }

    if not_empty(&req) {
        return HttpResponse::BadRequest().cookie(cookie).body("Campos incoretos");
    }

    if let Some(v) = results.1.c_producer {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("")
        }
    }

    let data = Farms {
        id: None,
        name: req.name.to_owned(),
        fields: req.fields.clone()
    };

    match farm_col().await.insert_one(data).await {
        Ok(result) => HttpResponse::Ok().cookie(cookie).json(result),
        Err(_) => HttpResponse::InternalServerError().body("")
    }
}

#[get("/farms")]
pub async fn get_farms(hed: HttpRequest) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => {
            return HttpResponse::new(s)
        }
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    if let Some(v) = results.1.c_user {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("Não permitido")
        }
    }
    
    let res_db = farm_col().await;

    let mut cursor = res_db
    .find(doc! {})
    .await
    .expect("Error");

    let mut producer: Vec<Farms> = Vec::new();

    while let Ok(Some(doc)) = cursor.try_next().await {
        producer.push(doc);
    } 

    HttpResponse::Ok()
        .cookie(cookie)
        .json(producer)
}

#[patch("/farm/{id}")]
pub async fn update_farm(req: Json<Farms>, hed: HttpRequest, id: web::Path<String>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<Farms>) -> bool {
        data.name.is_empty()
    }

    if not_empty(&req) {
        return HttpResponse::BadRequest().body("Requisição mal formatada.");
    }

    if let Some(v) = results.1.c_producer {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("")
        }
    }

    let obj_id = ObjectId::parse_str(id.into_inner()).unwrap();

    let filter = doc! { "_id":  obj_id};
    let update = doc! { "$set": doc! { 
        "name": req.name.clone()
    }};

    let res_db = farm_col().await;

    let res = res_db.update_one(filter, update).await;

    match res {
        Ok(value) => {
            HttpResponse::Ok()
                .cookie(cookie)
                .json(value)
        },
        Err(_) => {
            HttpResponse::BadRequest()
                .cookie(cookie)
                .body("Errro")
        }
    }
}