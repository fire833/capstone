use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use dashmap::DashMap;
use hyper::{Body, Request, Response, Server, Client, Uri, Method};
use hyper::service::{make_service_fn, service_fn};
use regex::Regex;
use lazy_static::lazy_static;

async fn handle(_req: Request<Body>, map: Arc<DashMap<String, String>>) -> Result<Response<Body>, hyper::Error> {
    println!("Got req path: {}", _req.uri());
    
    lazy_static! {
        static ref re: Regex = Regex::new(r"^/session/(.*)/").unwrap();
    }

    if !(_req.method() == Method::POST && _req.uri().path_and_query().unwrap().eq("/session")) {
        let path = _req.uri().path_and_query().unwrap();
        for captures in re.captures_iter(_req.uri().path_and_query().unwrap().path()) {
            println!("{:#?}", captures.get(1).unwrap().as_str());
        }
    } else {
        println!("Requesting new session");
    }

    let uri = Uri::builder().scheme("http").authority("192.168.49.2:30000").path_and_query(_req.uri().path_and_query().unwrap().to_string()).build().unwrap();
    let mut req = _req;
    *req.uri_mut() = uri;


    let client = Client::new();
    client.request(req).await
}


#[tokio::main]
async fn main() {
    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([0, 0, 0, 0], 6543));

    // And a MakeService to handle each connection...
    let session_id_to_endpoint = Arc::new(DashMap::<String, String>::new());


    // TODO: make sense of this utter mess
    let server = Server::bind(&addr).serve(
            make_service_fn(move |_con| {
            let map = session_id_to_endpoint.clone();
            async {
                Ok::<_, Infallible>(service_fn(move |_conn| {
                    handle(_conn, map.clone())
                }))
            }
        })
    );

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}