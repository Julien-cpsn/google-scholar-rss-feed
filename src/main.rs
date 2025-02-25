use google_scholar_query::scholar::{init_client, ScholarArgs};
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::http::Error;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use rss::{Category, Channel, ChannelBuilder, ItemBuilder, Source, TextInput};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use tokio::net::TcpListener;
use tokio::time::Instant;

lazy_static! {
    pub static ref RSS_CHANNELS: Arc<RwLock<HashMap<String, Channel>>> = Arc::new(RwLock::new(HashMap::new()));
}

#[tokio::main]
async fn main() {
    let address = env::args().nth(1).unwrap_or_else(|| "127.0.0.1:3005".to_string());

    let addr = SocketAddr::from_str(&address).unwrap();

    // We create a TcpListener and bind it to 127.0.0.1:3000
    println!("Listening on {address}...");
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("Server started");
    let mut last_update = Instant::now();
    loop {
        // Clear every day to avoid overflow
        if last_update.elapsed() >= Duration::from_secs(3600) {
            println!("Clearing cache");
            RSS_CHANNELS.write().clear();
            last_update = Instant::now();
        }

        let (stream, _) = listener.accept().await.unwrap();
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            match http1::Builder::new().serve_connection(io, service_fn(send_rss)).await {
                Ok(_) => (),
                Err(err) => eprintln!("Error serving connection: {:?}", err)
            }
        });
    }
}

async fn send_rss(request: Request<Incoming>) -> Result<Response<Full<Bytes>>, Error> {
    let params: HashMap<String, String> = request
        .uri()
        .query()
        .map(|v| {
            url::form_urlencoded::parse(v.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_else(HashMap::new);

    let response;

    if params.is_empty() || !params.contains_key("username") {
        response = Response::builder()
            .header("Access-Control-Allow-Origin", "*")
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from("no \"username\" param provided")))?;
    }
    else {
        let username = params.get("username").unwrap();
        let channel = generate_channel_if_needed(String::from(username)).await;

        response = Response::builder()
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .status(StatusCode::OK)
            .body(Full::new(Bytes::from(channel.to_string())))?;
    }

    Ok(response)
}

async fn generate_channel_if_needed(username: String) -> Channel {
    if !RSS_CHANNELS.read().contains_key(&username) {
        let mut new_channel = ChannelBuilder::default()
            .title(format!("{} scientific publications", username))
            .description(format!("An RSS feed for {} scientific publications. Parsed from Google Scholar.", username))
            .language(String::from("en-US"))
            .generator(String::from("google-scholar-rss-feed"))
            .copyright(String::from("© Google Scholar"))
            .ttl(String::from("60"))
            .docs(String::from("https://cyber.harvard.edu/rss/rss.html"))
            .text_input(TextInput {
                title: String::from("Google Scholar"),
                description: String::from("Search Google Scholar"),
                name: String::from("q"),
                link: String::from("https://scholar.google.com/scholar"),
            })
            .categories(vec![Category::from("Scientific Research")])
            .build();

        update_rss_channel(&username, &mut new_channel).await;

        RSS_CHANNELS.write().insert(username.clone(), new_channel);
    }

    RSS_CHANNELS.read().get(&username).unwrap().clone()
}

async fn update_rss_channel(username: &str, channel: &mut Channel) {
    println!("Updating RSS channel for \"{username}\"");

    let client = init_client();
    let query = ScholarArgs {
        query: format!("author:\"{}\"", username),
        cite_id: None,
        from_year: None,
        to_year: None,
        sort_by: None,
        cluster_id: None,
        lang: None,
        lang_limit: None,
        limit: Some(100),
        offset: None,
        adult_filtering: None,
        include_similar_results: None,
        include_citations: None,
    };

    let results = match client.scrape_scholar(Box::from(query)).await {
        Ok(results) => results,
        Err(_e) => panic!("Google scholar query failed"),
    };


    let mut items = vec![];

    for result in results {
        let source_url = match result.domain.contains(".") {
            true => result.domain.clone(),
            false => format!("{}.com", result.domain)
        };
        
        let item = ItemBuilder::default()
            .title(result.title)
            .author(result.author)
            .description(result.conference)
            .link(result.link)
            .source(Source {
                url: source_url,
                title: Some(String::from(&result.domain)),
            })
            .pub_date(result.year)
            .content(result.abs)
            .build();

        items.push(item);
    }


    channel.set_items(items);

    let now = Utc::now().to_rfc2822();
    channel.set_pub_date(now.clone());
    channel.set_last_build_date(now);
}