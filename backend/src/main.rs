use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;

mod db;
mod models;
mod config;
mod city_loader;
mod morphology_analyzer;
mod evolution_detector;
mod spatial_syntax;
mod fractal;
mod mann_kendall;
mod errors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/ancient_city".to_string());
    
    let pool = db::init_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    let port = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);

    println!("Server starting on http://localhost:{}", port);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .route("/api/health", web::get().to(city_loader::health_check))
            .route("/api/dynasties", web::get().to(city_loader::get_dynasties))
            .route("/api/sites", web::get().to(city_loader::get_city_sites))
            .route("/api/sites/{id}", web::get().to(city_loader::get_city_site_by_id))
            .route("/api/sites/dynasty/{dynasty_id}", web::get().to(city_loader::get_sites_by_dynasty))
            .route("/api/zones/{site_id}", web::get().to(city_loader::get_functional_zones))
            .route("/api/roads/{site_id}", web::get().to(city_loader::get_roads))
            .route("/api/buildings/{site_id}", web::get().to(city_loader::get_buildings))
            .route("/api/population/{site_id}", web::get().to(city_loader::get_population))
            .route("/api/morphology/{site_id}", web::get().to(morphology_analyzer::get_morphology))
            .route("/api/morphology/analyze/{site_id}", web::post().to(morphology_analyzer::analyze_morphology))
            .route("/api/syntax/roads/{site_id}", web::get().to(morphology_analyzer::get_road_syntax))
            .route("/api/trends/analyze", web::post().to(evolution_detector::analyze_trends))
            .route("/api/trends", web::get().to(evolution_detector::get_trends))
            .route("/api/compare", web::post().to(evolution_detector::compare_sites))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
