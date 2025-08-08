use actix_web::{cookie::Cookie, get, patch, post, web::{self, Json}, HttpRequest, HttpResponse};

use crate::{database::mongo_db::producer_col, models::property_model::Producer, service::session::authorization};

use mongodb::bson::{doc, oid::ObjectId};
use futures::TryStreamExt;


#[post("/producer")]
pub async fn created_producer(hed: HttpRequest, req: Json<Producer>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<Producer>) -> bool {
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

    let data = Producer {
        id: None,
        name: req.name.to_owned(),
        users: None,
        farms: None
    };

    match producer_col().await.insert_one(data).await {
        Ok(result) => HttpResponse::Ok().cookie(cookie).json(result),
        Err(_) => HttpResponse::InternalServerError().body("")
    }
}

#[get("/producers")]
pub async fn get_all_producer(hed: HttpRequest) -> HttpResponse {
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

    if let Some(v) = results.1.c_producer {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("Não permitido")
        }
    }
    
    let res_db = producer_col().await;

    let mut cursor = res_db
    .find(doc! {})
    .await
    .expect("Error");

    let mut producer: Vec<Producer> = Vec::new();

    while let Ok(Some(doc)) = cursor.try_next().await {
        producer.push(doc);
    } 

    HttpResponse::Ok()
        .cookie(cookie)
        .json(producer)
}

#[get("/producer/{id}")]
pub async fn get_producer(hed: HttpRequest, id: web::Path<String>) -> HttpResponse {
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

    let res_db = producer_col().await;
    
    let projection = doc! {
        "farms": false
    };

    let obj_id =  match ObjectId::parse_str(id.into_inner()) {
        Ok(v) => v,
        Err(_) => return HttpResponse::BadRequest().cookie(cookie).body("")
    };
    
    let filter = doc! { "_id":  obj_id};

    let res = res_db
    .find_one(filter)
    .projection(projection)
    .await
    .expect("Error");

    HttpResponse::Ok()
        .cookie(cookie)
        .json(res)
}

#[patch("/producer/{id}")]
pub async fn update_producer(req: Json<Producer>, hed: HttpRequest, id: web::Path<String>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<Producer>) -> bool {
        data.name.is_empty()
        || data.farms.is_none()
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
        "name": req.name.clone(),
        "users": req.users.clone(),
        "farms": req.farms.clone()
    }};

    let res_db = producer_col().await;

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