//! DNS server for the rusty coin
//! a DNS server will be responsible for query the existing nodes of a network
//! therefore, it should maintain a list of nodes that is current active in the network
//! and their IP addresses, it may also maintain those nodes that are not active anymore
//! the nodes should be able to:
//! - register themselves to the DNS server
//! - query the existing active nodes in the network
//!
//! API:
//! - GET /
//!     - the index page of the DNS server, test the server is running
//!     - return "Hello World!"
//! - POST /register
//!    - register a node with the DNS server
//!    - require `ipv4_address: String` and `port: u16` in the request body
//!    - return "register node <IP address>:<port> successfully"
//!    - otherwise, return 400 Bad Request
//! - POST /deregister
//!    - deregister a node from the DNS server
//!    - require `ipv4_address: String` and `port: u16` in the request body
//!    - return "deregister node <IP address>:<port> successfully"
//!    - otherwise, return 400 Bad Request
//! - GET /query
//!    - query the existing active nodes in the network
//!    - randomly poll a node from the list of active nodes
//!    - and return its IP address & port
//!    - return "no active nodes in the network" if there is no active nodes


use std::sync::{Arc, Mutex};
use actix_web::{App, HttpServer, Responder, get, HttpResponse, post, web};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use rand::prelude::SliceRandom;

#[get("/")]
async fn index() -> impl Responder {
    /// the index page of the DNS server
    HttpResponse::Ok().body("Hello World!")
}

#[post("/deregister")]
async fn deregister(info: web::Json<Node>) -> impl Responder {
    /// deregister a node from the DNS server

    NODES.lock().unwrap().retain(|node| {
        node.ipv4_address != info.ipv4_address || node.port != info.port
    });

    HttpResponse::Ok().body(format!("deregister node {}:{} successfully", info.ipv4_address, info.port))
}

#[post("/register")]
async fn register(info: web::Json<Node>) -> impl Responder {
    /// register a node with the DNS server
    let node = info.into_inner();
    let ipv4_address = node.ipv4_address;
    let port = node.port;
    let node = Node {
        ipv4_address: ipv4_address.clone(),
        ipv6_address: None,
        port,
    };
    NODES.lock().unwrap().push(node);

    HttpResponse::Ok().body(format!("register node {}:{} successfully", ipv4_address, port))
}

#[get("/query")]
async fn query() -> impl Responder {
    /// query the existing active nodes in the network
    /// randomly poll a node from the list of active nodes
    /// and return its IP address & port & public key
    let nodes = NODES.lock().unwrap();

    if nodes.is_empty() {
        return HttpResponse::Ok().json("no active nodes in the network");
    }

    let node = nodes.choose(&mut rand::thread_rng()).unwrap();
    let ipv4_address = node.ipv4_address.clone();
    let port = node.port;


    HttpResponse::Ok().json(Node {
        ipv4_address,
        ipv6_address: None,
        port,
    })
}

#[derive(Debug,Deserialize, Serialize)]
struct Node {
    /// a node in the network
    /// and an IP address, either IPv4 or IPv6
    ipv4_address: String,
    /// reserved for IPv6
    ipv6_address: Option<String>,
    /// and a port
    port: u16,

}

// the list of nodes that are currently active in the network
// the key is the address of the node
// the value is the public key of the node
static NODES: Lazy<Arc<Mutex<Vec<Node>>>> = Lazy::new(||Arc::new(Mutex::new(Vec::new())));


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    const DNS_SERVER_IP: &str = "127.0.0.1";
    const DNS_SERVER_PORT: u16 = 8080;

    println!("DNS server is listening on http://{}:{}", DNS_SERVER_IP, DNS_SERVER_PORT);
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(register)
            .service(deregister)
            .service(query)
    })
        .bind((DNS_SERVER_IP,DNS_SERVER_PORT))?
        .run()
        .await
}
