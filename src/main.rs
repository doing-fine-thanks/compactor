use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use env_logger;
use log::info;
use rand::seq::IteratorRandom;
use reqwest::{header::USER_AGENT, StatusCode};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use urlencoding::decode;

const DUCK_DUCK_GO_TEMPLATE: &str = "https://html.duckduckgo.com/html/?q=";
static NO_LIST: &'static [&str] = &["usnews", "tennessean"];

#[derive(Deserialize)]
struct UrlFetch {
    url: String,
}

#[derive(Deserialize)]
struct SearchTerm {
    term: String,
}

#[get("/echo")]
async fn echo(url_fetch: web::Query<UrlFetch>) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client.get(url_fetch.url.clone()).header(USER_AGENT, "rust web-api-client demo").send().await?;

    Ok(format!("{:#?}", response.text().await?))
}

#[get("/complect")]
async fn complect(search_term: web::Query<SearchTerm>) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(DUCK_DUCK_GO_TEMPLATE.to_owned() + &search_term.term)
        .header(USER_AGENT, "chrome")
        .send()
        .await?;

    let document = Html::parse_document(response.text().await?.as_str());
    let selector = Selector::parse("#links > div > div > h2 > a").unwrap();

    let mut combined_document: Vec<String> = Vec::new();
    let subdocument = document
        .select(&selector)
        .map(|x| x.value().attr("href"))
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .map(|x| decode(x).unwrap().into_owned())
        .map(|x| x.split("uddg=").last().unwrap().split("&rut=").collect::<Vec<&str>>()[0].to_owned());

    for url in subdocument {
        if NO_LIST.contains(&url.split(".").collect::<Vec<&str>>()[1]) {
            continue;
        }
        let url_response = client.get(url).header(USER_AGENT, "chrome").send().await?;
        let sub_document: Html = Html::parse_document(url_response.text().await?.as_str());
        let mut rng = rand::thread_rng();
        for node in sub_document.root_element().traverse().into_iter().choose_multiple(&mut rng, 15) {
            match node {
                ego_tree::iter::Edge::Open(node_ref) => {
                    if let Some(element) = ElementRef::wrap(node_ref) {
                        combined_document.push(element.html().clone());
                        ()
                    }
                }
                _ => (),
            }
        }
    }

    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(combined_document.to_owned().join("\n")))
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1";
    let port = 8080;
    env_logger::init_from_env(env_logger::Env::new());

    info!(" 🌐 will server on: http://{}:{} 🌐 ", addr, port);
    HttpServer::new(|| App::new().service(index).service(echo).service(complect).wrap(Logger::default()))
        .bind((addr, port))?
        .run()
        .await
}
