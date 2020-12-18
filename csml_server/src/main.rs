use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer, http::header};
use actix_files as fs;

mod routes;

const MAX_BODY_SIZE: usize = 8_388_608; // 8MB

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
  std::env::set_var("RUST_LOG", "actix_web=info");
  env_logger::init();

  let server_port: String = match std::env::var("ENGINE_SERVER_PORT") {
    Ok(val) => val,
    Err(_) => "5000".to_owned(),
  };
  println!("CSML Server listening on port {}", server_port);

  HttpServer::new(|| {
    App::new()
      .wrap(
        Cors::new()
          .send_wildcard()
          .allowed_methods(vec!["GET", "POST"])
          .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE
          ])
          .max_age(86_400) //24h
          .finish(),
      )
      .wrap(middleware::Logger::default())
      .data(web::JsonConfig::default().limit(MAX_BODY_SIZE))

      .service(fs::Files::new("/static", "./static").use_last_modified(true))

      .service(routes::index::home)
      .service(routes::validate::handler)
      .service(routes::run::handler)
      .service(routes::sns::handler)
      .service(routes::create_bot::handler)
      .service(routes::get_bot_by_version_id::handler)
      .service(routes::get_last_bot_version::handler)
      .service(routes::get_bot_versions::handler)

      .service(routes::conversations::get_open)
      .service(routes::conversations::close_user_conversations)

  })
  .bind(format!("0.0.0.0:{}", server_port))?
  .run()
  .await
}

