use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

use super::model::{Threads, ThreadsError};

#[get("/threads/{id}")]
async fn threads(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let id = id.into_inner();
    match Threads::fetch(id, db_pool.get_ref()).await {
        Ok(data) => HttpResponse::Ok()
            .content_type("application/json; charset=utf-8")
            .json(data),
        Err(e) => {
            println!("/threads/{:?}: {:?}", id, e);
            match e {
                ThreadsError::DatabaseError { .. } => {
                    HttpResponse::InternalServerError().json(None::<String>)
                }
            }
        }
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(threads);
}
