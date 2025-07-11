
use actix_web::http::StatusCode;
use mongodb::error::{Error, ErrorKind, WriteError, WriteFailure};


pub fn erro_db(err: Error) -> (StatusCode, String) {
    if let ErrorKind::Write(WriteFailure::WriteError(WriteError { code, ..})) = *err.kind {
        match code {
            11000 => (StatusCode::CONFLICT, "Dados existentes".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Code de erro desconhecido".to_string())
        }
        
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "Erro desconheciddo".to_string())
    }
}