use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Result};
use chashmap::CHashMap;
use handlebars::Handlebars;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::{collections::HashMap, error::Error, sync::Arc};

#[derive(Deserialize)]
struct FormData {
    target: String,
}

struct AppState<'a> {
    templates: Handlebars<'a>,
    url_map: Arc<CHashMap<String, String>>,
    url_map_reverse: Arc<CHashMap<String, String>>,
}

#[get("/")]
async fn index() -> Result<NamedFile, std::io::Error> {
    NamedFile::open("static/index.html")
}

#[get("/{target:[a-zA-Z0-9]{5}}")]
async fn redirect<'a>(
    state: web::Data<AppState<'a>>,
    target: web::Path<String>,
) -> Result<web::Redirect> {
    let href = state
        .url_map
        .get(&target.into_inner())
        .ok_or(actix_web::error::ErrorNotFound(""))?;
    Ok(web::Redirect::to(href.to_string()).temporary())
}

fn make_short_name() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(5)
        .map(char::from)
        .collect()
}

#[post("/")]
async fn post_index<'a>(
    state: web::Data<AppState<'a>>,
    data: web::Form<FormData>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let mut template_data = HashMap::new();

    if let Some(name) = state.url_map_reverse.get(&data.target) {
        template_data.insert("target", name.to_string());
    } else {
        let name = make_short_name();

        state.url_map.insert(name.clone(), data.target.clone());
        state
            .url_map_reverse
            .insert(data.target.clone(), name.clone());
        template_data.insert("target", name);
    }

    let s = state.templates.render("posted", &template_data)?;
    Ok(HttpResponse::Ok().body(s))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let url_map = Arc::new(CHashMap::new());
    let url_map_reverse = Arc::new(CHashMap::new());

    env_logger::init();

    HttpServer::new(move || {
        let mut templates = Handlebars::new();
        templates
            .register_templates_directory(".hbs", "templates")
            .unwrap();

        let data = AppState {
            templates,
            url_map: url_map.clone(),
            url_map_reverse: url_map_reverse.clone(),
        };

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
