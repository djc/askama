//use actix_files as fs;
use actix_web::{http, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Result};
use askama::Template;

#[derive(Template)]
#[template(path = "home/home-en.html")]
struct HomeEn<'a> {
    lang: &'a str,
    title: &'a str,
    page: &'a str,
}

#[derive(Template)]
#[template(path = "home/home-it.html")]
struct HomeIt<'a> {
    lang: &'a str,
    title: &'a str,
    page: &'a str,
}

#[derive(Template)]
#[template(path = "about/about-en.html")]
struct AboutEn<'a> {
    lang: &'a str,
    title: &'a str,
    page: &'a str,
}

#[derive(Template)]
#[template(path = "about/about-it.html")]
struct AboutIt<'a> {
    lang: &'a str,
    title: &'a str,
    page: &'a str,
}

async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::TemporaryRedirect()
        .insert_header((http::header::LOCATION, "/en"))
        .finish())
}

async fn home(req: HttpRequest) -> Result<HttpResponse> {
    let lang: String = req.match_info().get("lang").unwrap().parse().unwrap();
    let s = match lang.as_str() {
        "en" => HomeEn {
            lang: &lang,
            title: &format!("Home-{}", &lang),
            page: &"home".to_string(),
        }
        .render()
        .unwrap(),
        "it" => HomeIt {
            lang: &lang,
            title: &format!("Home-{}", &lang),
            page: &"home".to_string(),
        }
        .render()
        .unwrap(),
        _ => "".to_string(),
    };
    if s == "" {
        Ok(HttpResponse::TemporaryRedirect()
            .insert_header((http::header::LOCATION, "/en"))
            .finish())
    } else {
        Ok(HttpResponse::Ok().content_type("text/html").body(s))
    }
}

async fn about(req: HttpRequest) -> Result<HttpResponse> {
    let lang: String = req.match_info().get("lang").unwrap().parse().unwrap();
    let s = match lang.as_str() {
        "en" => AboutEn {
            lang: &lang,
            title: &format!("About-{}", &lang),
            page: &"about".to_string(),
        }
        .render()
        .unwrap(),
        "it" => AboutIt {
            lang: &lang,
            title: &format!("About-{}", &lang),
            page: &"about".to_string(),
        }
        .render()
        .unwrap(),
        _ => "".to_string(),
    };
    if s == "" {
        Ok(HttpResponse::TemporaryRedirect()
            .insert_header((http::header::LOCATION, "/en"))
            .finish())
    } else {
        Ok(HttpResponse::Ok().content_type("text/html").body(s))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // start http server
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            //.service(fs::Files::new("/static", "static"))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/{lang}").route(web::get().to(home)))
            .service(web::resource("/{lang}/about").route(web::get().to(about)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
