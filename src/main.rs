use actix_files::{NamedFile, Files};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use chashmap::CHashMap;
use handlebars::Handlebars;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use serde::Deserialize;
use std::{collections::HashMap, sync::Mutex, sync::Arc, error::Error};

#[derive(Deserialize)]
struct FormData {
    target: String,
}

struct AppState<'a> {
  hb: Handlebars<'a>,
  h: Arc<CHashMap<String, String>>
}

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
  NamedFile::open("static/index.html")
}

#[get("/{target:[a-zA-Z0-9]{5}}")]
async fn redirect<'a>(state: web::Data<AppState<'a>>, target: web::Path<String>) -> Result<web::Redirect> {
  println!("Redirect");
  let href = state.h.get(&target.into_inner()).ok_or(actix_web::error::ErrorNotFound(""))?;
  Ok(web::Redirect::to(href.to_string()).temporary())
}

#[post("/")]
async fn post_index<'a>(state: web::Data<AppState<'a>>, data: web::Form<FormData>) -> Result<HttpResponse, Box<dyn Error>> {
  let name = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(5)
    .map(char::from)
    .collect::<String>();
  state.h.insert(name.clone(), data.target.clone());
  let mut yeet = HashMap::new();
  yeet.insert("target", &name);
  println!("{:?}", yeet);
  let s = state.hb.render("posted", &yeet)?;
  Ok(HttpResponse::Ok().body(s))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  let h = Arc::new(CHashMap::new());
  HttpServer::new(move || {
    let mut hb = Handlebars::new();
    hb.register_templates_directory(".hbs", "templates").unwrap();

    let data = AppState{hb, h: h.clone()};

    App::new()
      .app_data(web::Data::new(data))
      .service(index)
      .service(post_index)
      .service(redirect)
      .service(Files::new("/static", "static"))
  })
  .bind(("127.0.0.1", 8080))?
  .run()
  .await
}
