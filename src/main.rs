extern crate futures;
extern crate hyper;
extern crate url;

use std::env;
use std::io::Error;

use futures::Stream;
use futures::future::{Future, ok};
use hyper::{Method, Uri};
use hyper::server::{Http, Request, Response, Service};

struct DebugRequest {
    url: Uri,
    method: Method,
    body: String,
}

use std::fmt;
use std::fmt::Display;

impl Display for DebugRequest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "url = {}\nmethod = {}\nbody = \"{}\"", self.url, self.method, self.body)
    }
}

fn log(uri: Uri, method: Method, body: String) -> Box<Future<Item = (), Error = Error>> {
    Box::new(ok(println!("{}", DebugRequest {
        url: uri,
        method: method,
        body: body,
    })))
}

struct Debug;

impl Service for Debug {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let method = req.method().clone();
        let uri = req.uri().clone();

        Box::new(
            req.body().concat2()
                .and_then(|chunks| ok(
                    String::from_utf8(
                        chunks.iter().map(|c| *c).collect::<Vec<u8>>()
                    ).unwrap_or("".to_owned())
                ))
                .and_then(move |body| log(uri, method, body).map_err(hyper::Error::from))
                .and_then(|_| ok(
                    Response::new().with_status(hyper::StatusCode::Ok)
                ))
        )
    }
}

fn main() {
    let port = env::var("DEBUG_PORT").unwrap_or("8080".to_owned());
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    println!("listening on {}", addr);
    Http::new().bind(&addr, || Ok(Debug)).and_then(|s| s.run()).unwrap();
}
