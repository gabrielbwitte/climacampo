use actix_web::cookie::Cookie;
use actix_web::HttpRequest;
use actix_web::{get, post, delete,web::Json, HttpResponse};
use futures::TryStreamExt;
use mongodb::bson::doc;
use crate::database::mongo_db::{session, user_col};
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
        Ok(v) => {
            let result = authentication(req.password.clone(), v).await;
            match result {
                Ok(v) => {
                    let cookie = Cookie::build("token", v)
                        .finish();
                        HttpResponse::Ok()
                            .cookie(cookie)
                            .finish()
                },
                Err(s) => HttpResponse::new(s),
            }
        }
        Err(_) => HttpResponse::InternalServerError().body("Error interno")
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

    let res_db = session().await;
    let res = res_db.delete_one(doc).await;

    match res {
        Ok(v) => HttpResponse::Accepted().json(v),
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

    let cookie = Cookie::build("token", token).finish();
    HttpResponse::Ok()
        .cookie(cookie)
        .json(users)
}

#[post("/user")]
pub async fn created_user(hed: HttpRequest, req: Json<User>) -> HttpResponse {
    fn not_empty(data: &Json<User>) -> bool {
        data.name.is_empty() 
            || data.username.is_empty() 
            || data.email.is_empty() 
            || data.password.is_empty()
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
        access: Access {
            c_d_user: req.access.c_d_user,
            get_users: req.access.get_users
        }
    };

    let res_db = match user_col().await {
        Ok(v) => v,
        Err(s) => return HttpResponse::new(s)
    };
       
    let res = res_db.insert_one(data).await;

    match res {
        Ok(r) => {
            let cookie = Cookie::build("token", token).finish();
            HttpResponse::Ok()
                .cookie(cookie)
                .json(r)
                },
        Err(e) => {
            let erro = erro_db(e.clone());
            let cookie = Cookie::build("token", token).finish();
            HttpResponse::build(erro.0)
                .cookie(cookie)
                .json(erro.1)
        }
    }
}