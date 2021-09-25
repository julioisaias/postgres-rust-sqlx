use crate::users::{ User, UserRequest, PaginationRequest };
use actix_web::{ delete, get, post, put, web, HttpResponse, Responder };
use sqlx::PgPool;

#[get("/users")]
async fn find_all(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = User::find_all(db_pool.get_ref()).await;
    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        _ => HttpResponse::BadRequest()
            .body("Error trying to read all users from database"),
    }
}

#[get("/users/")]
async fn find_all_pagintation(db_pool: web::Data<PgPool>, web::Query(info): web::Query<PaginationRequest> ) -> impl Responder {
    let filter = info.filter;
    let sort_column = info.sortcolumn;
    let sort_order = info.sortorder;

    let result = User::find_all_with_pagination(db_pool.get_ref(), &filter, &sort_column, &sort_order).await;
    match result {
        Ok(users) => HttpResponse::Ok().json(users),
        _ => HttpResponse::BadRequest()
            .body("Error trying to read all users like table from database"),
    }
}

#[get("/user/{id}")]
async fn find(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = User::find_by_id(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("User not found"),
    }
}

#[post("/user")]
async fn create(
    user: web::Json<UserRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result = User::create(user.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("Error trying to create new user"),
    }
}

#[put("/user/{id}")]
async fn update(
    id: web::Path<i32>,
    user: web::Json<UserRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    let result =
        User::update(id.into_inner(), user.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(user) => HttpResponse::Ok().json(user),
        _ => HttpResponse::BadRequest().body("User not found"),
    }
}

#[delete("/user/{id}")]
async fn delete(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    let result = User::delete(id.into_inner(), db_pool.get_ref()).await;
    match result {
        Ok(rows) => {
            if rows > 0 {
                HttpResponse::Ok()
                    .body(format!("Successfully deleted {} record(s)", rows))
            } else {
                HttpResponse::BadRequest().body("User not found")
            }
        }
        _ => HttpResponse::BadRequest().body("User not found"),
    }
}

// function that will be called on new Application to configure routes for this module
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all);
    cfg.service(find_all_pagintation);
    cfg.service(find);
    cfg.service(create);
    cfg.service(update);
    cfg.service(delete);
}
