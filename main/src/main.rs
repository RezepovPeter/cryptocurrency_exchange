use actix_web::{ web, HttpServer, App };
use utils::init_db;

mod handlers;
mod models;
mod db;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let database_url = "postgres://postgres:password@db:5432/prak_3";
    init_db().await.unwrap();
    HttpServer::new(move || {
        App::new()
            .route("/user", web::post().to(handlers::create_user)) // POST
            .route("/order", web::post().to(handlers::create_order)) // POST
            .route("/order", web::get().to(handlers::get_orders)) // GET
            .route("/order", web::delete().to(handlers::delete_order)) // DELETE
            .route("/lot", web::get().to(handlers::get_lots)) // GET
            .route("/pair", web::get().to(handlers::get_pairs)) // GET
            .route("/balance", web::get().to(handlers::get_balance)) // GET
    })
        .bind("0.0.0.0:1338")?
        .run().await
}
