#[macro_use]
extern crate log;

use actix_web::{ web, App, HttpResponse, HttpServer, Responder };
use actix_cors::Cors;
use anyhow::Result;
use dotenv::dotenv;
use listenfd::ListenFd;
use sqlx::PgPool;
use std::env;


mod users;

async fn index() -> impl Responder {
    HttpResponse::Ok().body(r#""#)
}

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut listenfd = ListenFd::from_env();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
        
    let db_pool = PgPool::connect(&database_url).await?;

    let mut server = HttpServer::new(move || {
        App::new()
            .data(db_pool.clone()) 
            .route("/", web::get().to(index))
            .configure(users::init) // init user routes
            .wrap(Cors::default().allow_any_origin())
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("HOST is not set in .env file");
            let port = env::var("PORT").expect("PORT is not set in .env file");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    info!("Starting server");
    server.run().await?;

    Ok(())
}
