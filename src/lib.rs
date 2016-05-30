extern crate iron;

use iron::prelude::*;
use iron::{Handler};
use iron::status;

struct ImageHandler {
}

impl ImageHandler {
    fn new() -> Self {
        return ImageHandler {};
    }
}

impl Handler for ImageHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let mut body = String::new();
        println!("{}", req.url.path.len());
        for elt in &req.url.path {
            println!("{}", elt);
            body.push_str("/");
            body.push_str(elt);
        }
        return Ok(Response::with((status::Ok, "Ahoy Telephone")));
    }
}

#[test]
fn it_works() {
}
