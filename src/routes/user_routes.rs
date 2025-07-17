use actix_web::HttpRequest;
use actix_web::{get, post, web::Json, HttpResponse};
use futures::TryStreamExt;
use mongodb::bson::doc;
use crate::database::mongo_db::user_col;
use crate::models::user_model::{Login, User, ObjJsonResponse};
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
        Ok(v) => {
            let result = authentication(req.password.clone(), v).await;
            match result {
                Ok(v) => {
                    let obj = ObjJsonResponse {
                        data: "".to_string(),
                        token: v
                    };
                        HttpResponse::Ok().json(obj)
                },
                Err(s) => HttpResponse::new(s),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Error interno")
    }
}

#[get("/users")]
pub async fn get_user(hed: HttpRequest) -> HttpResponse {
    let session = authorization(hed).await;

    let token = match session {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
    
    let mut cursor = res_db
    .find(doc! {})
    .await
    .ok()
    .expect("Error");

    let mut users: Vec<User> = Vec::new();

    while let Some(doc) = cursor.try_next().await.unwrap() {
        users.push(doc);
    }

    let obj = ObjJsonResponse {
        data: users,
        token: token
    };
    HttpResponse::Ok()
        .json(obj)
}

#[post("/user")]
pub async fn created_user(hed: HttpRequest, req: Json<User>) -> HttpResponse {
    fn not_empty(data: &Json<User>) -> bool {
        data.name.is_empty() 
            || data.username.is_empty() 
            || data.email.is_empty() 
            || data.password.is_empty() 
            || data.access.is_empty()
    }

    if not_empty(&req) {
        return HttpResponse::BadRequest().body("Campos incoretos");
    }

    let session = authorization(hed).await;

    let token = match session {
        Ok(t) => t,
        Err(s) => return HttpResponse::new(s)
    };

    let hashded = created_hash(req.password.clone()).await;
    let password_hash = match hashded {
        Ok(v) => v,
        Err(_) => return  HttpResponse::InternalServerError().body("Error ao gravar senha")
    };

    let data = User {
        id: None,
        name: req.name.to_owned(),
        username: req.username.to_owned(),
        email: req.email.to_owned(),
        password: password_hash,
        access: req.access.to_owned(),
    };

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
       
    let res = res_db.insert_one(data).await;

    match res {
        Ok(r) => {
            let obj = ObjJsonResponse {
                data: r,
                token: token
            };
            HttpResponse::Ok()
                .json(obj)
        },
        Err(e) => {
            let erro = erro_db(e.clone());
            let obj = ObjJsonResponse {
                data: erro.1,
                token: token
            };
            HttpResponse::build(erro.0)
                .json(obj)

        }
    }
   
}