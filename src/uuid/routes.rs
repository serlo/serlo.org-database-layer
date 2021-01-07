use crate::uuid::Uuid;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::MySqlPool;

#[get("/uuid/{id}")]
async fn find(id: web::Path<i32>, db_pool: web::Data<MySqlPool>) -> impl Responder {
    let result = Uuid::find_by_id(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(uuid) => HttpResponse::Ok().json(uuid),
        _ => HttpResponse::NotFound().body("UUID not found"),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find);
}