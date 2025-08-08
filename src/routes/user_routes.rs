use actix_web::cookie::Cookie;
use actix_web::{web, HttpRequest};
use actix_web::{get, post, delete, patch, web::Json, HttpResponse};
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use crate::database::mongo_db::{producer_col, session_col, user_col};
use crate::models::property_model::Producer;
use crate::models::user_model::{Access, Login, User};
use crate::service::session::{created_hash, authentication, authorization};
use crate::database::error_db::{erro_db};

#[post("/login")]
pub async fn login(req: Json<Login>) -> HttpResponse {
    let doc = doc! {"username": &req.username};

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
    let res = res_db.find_one(doc).await;
    match res {
        Ok(v_user) => {                
                let result = authentication(req.password.clone(), v_user).await;
                match result {
                    Ok(v) => {
                        let cookie = Cookie::build("token", v.0)
                        .path("/")
                        .secure(false)
                        .http_only(false)
                        .finish();
                    HttpResponse::Ok()
                    .cookie(cookie)
                    .json(v.1)
                },
                Err(s) => HttpResponse::new(s),
                }  
        }
        Err(e) => {
            println!("{:?}",e);
            HttpResponse::InternalServerError().body("Error interno !")
        }
    }
}

#[delete("/logoff")]
pub async fn logoff(hed: HttpRequest) -> HttpResponse {

    let get_token = hed.headers().get("token");
    let result_token = match get_token {
        Some(v) => v.to_str(),
        None =>  return HttpResponse::BadRequest().body("Token não encontrado")
    };
    let token = match result_token {
        Ok(v) => v.to_string(),
        Err(_) => return HttpResponse::BadRequest().body("Conteudo mal formatado")
    };

    let doc = doc! { "token": token };

    let res_db = session_col().await;
    let res = res_db.delete_one(doc).await;

    match res {
        Ok(v) => HttpResponse::Accepted().json(v),
        Err(_) => HttpResponse::InternalServerError().body("Error interno")
    }
}

#[get("/user/profile/{id}")]
pub async fn get_user_profile(hed: HttpRequest, id: web::Path<String>) -> HttpResponse {
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

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
    
    let projection = doc! {
        "username": false,
        "password": false,
        "access": {
            "c_user": false,
            "c_access": false,
            "c_producer": false
        }
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

    #[derive(Debug, Serialize, Deserialize)]
    struct Res {
        user: User,
        producer: Vec<Producer>
    }

    let filter_producer = doc! {
        "users": obj_id
    };
    let projection_producer = doc! {
    "users": false,
    "farms": false
    };
    let mut cursor_producer = producer_col()
        .await
        .find(filter_producer)
        .projection(projection_producer)
        .await
        .expect("Error");

    let mut producers: Vec<Producer> = Vec::new();

    while let Ok(Some(doc)) = cursor_producer.try_next().await {
        producers.push(doc);
    }

    let response = Res {
        user: res.expect(""),
        producer: producers
    };
    HttpResponse::Ok()
        .cookie(cookie)
        .json(response)
}

#[get("/users")]
pub async fn get_all_users(hed: HttpRequest) -> HttpResponse {
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

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
    
    if let Some(v) = results.1.c_user {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("Não permitido")
        }
    }

    let projection = doc! {
        "username": false,
        "password": false
    };

    let mut cursor = res_db
    .find(doc! {})
    .projection(projection)
    .await
    .expect("Error");

    let mut users: Vec<User> = Vec::new();

    while let Ok(Some(doc)) = cursor.try_next().await {
        users.push(doc);
    } 

    HttpResponse::Ok()
        .cookie(cookie)
        .json(users)
}

#[post("/user")]
pub async fn created_user(hed: HttpRequest, req: Json<User>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<User>) -> bool {
        data.name.is_empty()
            || data.username.is_none()
            || data.email.is_empty()
            || data.password.is_none()
            || data.access.c_user.is_none()
            || data.access.c_access.is_none()
            || data.access.c_producer.is_none()
            || data.access.modules.is_none()
    }

    if not_empty(&req) {
        return HttpResponse::BadRequest().cookie(cookie).body("Campos incoretos");
    }

    if let Some(v) = results.1.c_user {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("")
        }
    }

    let password = match req.password.clone() {
        Some(v) => v,
        None => return HttpResponse::BadRequest().body("Campo incoreto")
    };

    let hashded = created_hash(password).await;
    let password_hash = match hashded {
        Ok(v) => v,
        Err(_) => return  HttpResponse::InternalServerError().body("Error ao gravar senha")
    };

    let data = User {
        id: None,
        name: req.name.to_owned(),
        username: req.username.to_owned(),
        email: req.email.to_owned(),
        password: Some(password_hash),
        access: Access { 
            c_user: req.access.c_user,
            c_access: req.access.c_access, 
            c_producer: req.access.c_producer,
            modules: req.access.modules.clone()
        }
    };

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
       
    let res = res_db.insert_one(data).await;

    match res {
        Ok(r) => {
            HttpResponse::Ok()
                .cookie(cookie)
                .json(r)
                },
        Err(e) => {
            let erro = erro_db(e.clone());
            HttpResponse::build(erro.0)
                .cookie(cookie)
                .json(erro.1)
        }
    }
}

#[patch("/user/access/{id}")]
pub async fn update_access_user(req: Json<Access>, hed: HttpRequest, id: web::Path<String>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(false)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<Access>) -> bool {
        data.c_user.is_none()
        || data.c_access.is_none()
        || data.c_producer.is_none()
        || data.modules.is_none()
    }

    if not_empty(&req) {
        return HttpResponse::BadRequest().body("Requisição mal formatada.");
    }

    if let Some(v) = results.1.c_access {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("")
        }
    }

    let obj_id = ObjectId::parse_str(id.into_inner()).unwrap();

    let filter = doc! { "_id":  obj_id};
    let update = doc! { "$set": doc! { "access": {
        "c_user": req.c_user,
        "c_access": req.c_access,
        "c_producer": req.c_producer,
        "modules": req.modules.clone()
    }}};

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };

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

#[patch("/user/profile/{id}")]
pub async fn update_profile_user(req: Json<User>, hed: HttpRequest, id: web::Path<String>) -> HttpResponse {
    let results = match authorization(hed).await {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };
    let cookie = Cookie::build("token", results.0)
        .path("/")
        .secure(true)
        .http_only(true)
        .finish();

    fn not_empty(data: &Json<User>) -> bool {
        data.name.is_empty()
        || data.username.is_none()
        || data.email.is_empty()
    }
    
    if not_empty(&req) {
        return HttpResponse::BadRequest().body("Requisição mal formatada.");
    }

    if let Some(v) = results.1.c_user {
        if !v {
            return HttpResponse::MethodNotAllowed().cookie(cookie).body("")
        }
    }

    let obj_id = ObjectId::parse_str(id.into_inner()).unwrap();

    let filter = doc! { "_id":  obj_id};
    let update = doc! { "$set": doc! { 
        "name": req.name.to_owned(),
        "username": req.username.to_owned(),
        "email": req.email.to_owned()
    } };

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };

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