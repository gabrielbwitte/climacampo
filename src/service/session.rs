extern crate bcrypt;

use actix_web::http::StatusCode;
use actix_web::HttpRequest;
use bcrypt::{hash, verify};
use chrono::Utc;
use mongodb::bson::{doc};

use uuid::Uuid;

use crate::database::mongo_db::session;
use crate::models::user_model::{Access, Session, User};

const EXPIRATION_IN_MILLISECONDS: i64 = 1296000000;

pub async fn created_hash(password: String) -> Result<String, StatusCode> {
    let hashded = hash(password, 12);

    match hashded {
        Ok(v) => Ok(v),
        Err(_) => {
            println!("Err hashded");
            Err(StatusCode::INTERNAL_SERVER_ERROR)}
    }
}

async fn created_session(doc_session: Session) -> Result<String, StatusCode> {

    let db = session().await;
    let filter = doc! { "user_id": &doc_session.user_id };
    let result_db_e = db.find_one(filter.clone()).await;
    let result_db = match result_db_e {
        Ok(v) => v,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    let token = doc_session.token.clone();
    match result_db {
        Some(_) => {
            let update_d = doc! { "$set": doc! {
                "token": &token,
                "start_date": doc_session.start_date
            } };
            let update_s = db.update_one(filter, update_d).await;
            match update_s {
                Ok(_) => Ok(token),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        },
        None => {
            let insert_s = db.insert_one(doc_session).await;
            match insert_s {
                Ok(_) => Ok(token),
                Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

pub async fn authentication(password: String, user_db: Option<User>) -> Result<String, StatusCode> {
    let result = match user_db {
        Some(v) => v,
        None => return Err(StatusCode::UNAUTHORIZED)
    };

    let password_db = match result.password {
        Some(v) => v,
        None => return Err(StatusCode::BAD_REQUEST)
    };

    let valid = verify(password, password_db.as_str());

    match valid {
        Ok(v) => {
            if v {
                let user_id = match result.id {
                    Some(v) => v,
                    None => return Err(StatusCode::PARTIAL_CONTENT)
                };

                let doc= Session {
                    id: None,
                    user_id,
                    token: Uuid::new_v4().to_string(),
                    start_date: Utc::now().timestamp_millis(),
                    access: Access { 
                        profile: result.access.profile,
                        c_d_user: result.access.c_d_user, 
                        get_users: result.access.get_users, 
                        climate: result.access.climate,
                        c_access: result.access.c_access,
                        mapa: result.access.mapa
                    }
                };
                created_session(doc).await
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

async fn renew_session(doc_session: Session) -> Result<String, StatusCode> {
    let db = session().await;
    let filter = doc! { "user_id": doc_session.user_id };

    let new_token = Uuid::new_v4().to_string();
    let new_start_date = Utc::now().timestamp_millis();

    let update_d = doc! { "$set": doc! {
        "token": &new_token,
        "start_date": new_start_date
    } };

    let update_s = db.update_one(filter, update_d).await;
    match update_s {
        Ok(_) => Ok(new_token),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

pub async fn authorization(headers: HttpRequest) -> Result<(String, Access), StatusCode> {
    let get_token = headers.headers().get("token");
    let result_token = match get_token {
        Some(v) => v.to_str(),
        None =>  return Err(StatusCode::BAD_REQUEST)
    };
    let token = match result_token {
        Ok(v) => v.to_string(),
        Err(_) => return Err(StatusCode::BAD_REQUEST)
    };

    let doc = doc! { "token": token };

    let res_db = session().await;
    let res = res_db.find_one(doc).await;
    
    match res {
        Ok(v) => {
            match v {
                Some(s) => {
                    let time_current: i64 = Utc::now().timestamp_millis() - s.start_date;
                    if time_current < EXPIRATION_IN_MILLISECONDS {
                        let access = Access {
                            profile: s.access.profile,
                            c_d_user: s.access.c_d_user,
                            get_users: s.access.get_users,
                            climate: s.access.climate,
                            c_access: s.access.c_access,
                            mapa: s.access.mapa
                        };
                        let renew_s = renew_session(s).await;
                        match renew_s {
                            Ok(token) => Ok((token, access)),
                            Err(s) => Err(s)
                        }
                    } else {
                        Err(StatusCode::UNAUTHORIZED)
                    }
                },
                None => Err(StatusCode::UNAUTHORIZED)
            }
        },
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}