use actix_session::{storage::RedisSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::dev::Server;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use dotenv::dotenv;
use std::env;
use std::time::Duration;

mod db;
mod game;
mod models;
mod routes;
mod rsa;
mod schema;
mod twentyfortyeight;

use crate::routes::*;
use crate::rsa::*;
use crate::twentyfortyeight::*;
use std::net::TcpListener;

type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn run(
    redis_store: RedisSessionStore,
    listener: TcpListener,
    pool: DbPool,
) -> std::io::Result<Server> {
    println!("Here");
    dotenv().ok();

    std::env::set_var("RUST_LOG", "actix_web=info");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: DbPool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let secret_key = Key::generate();

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                redis_store.clone(),
                secret_key.clone(),
            ))
            .app_data(web::Data::new(pool.clone()))
            .route("/health_check", web::get().to(health_check))
            .route("/generate_keys", web::get().to(generate_rsa_keys))
            .service(
                web::scope("/api")
                    .route("/getProjects", web::get().to(get_projects))
                    .route("/getSkills", web::get().to(get_skills))
                    .route("/getBio", web::get().to(get_bio))
                    .route("/chat", web::post().to(chat)),
            )
            .service(
                web::scope("/game")
                    .route("/new", web::post().to(new_game))
                    .route("/move", web::post().to(make_move))
                    .route("/undo", web::post().to(undo))
                    .route("/reset", web::post().to(reset)),
            )
            .service(actix_files::Files::new("/", "./client/public").index_file("index.html"))
            .default_service(web::route().to(move |req: HttpRequest| {
                let path = req.path().to_owned();
                async move {
                    if path.starts_with("/api") || path.starts_with("/game") {
                        HttpResponse::NotFound().finish()
                    } else {
                        match actix_files::NamedFile::open("./client/public/index.html") {
                            Ok(file) => file.into_response(&req),
                            Err(_) => HttpResponse::NotFound().finish(),
                        }
                    }
                }
            }))
    })
    .listen(listener)?
    .client_request_timeout(Duration::new(5 * 60, 0))
    .run();
    Ok(server)
}
//.bind(format!(
//    "0.0.0.0:{}",
//    env::var("PORT").unwrap_or_else(|_| "5000".to_string())
//))?
