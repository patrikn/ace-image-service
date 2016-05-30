#[macro_use] extern crate iron;
extern crate hyper;
extern crate bodyparser;
extern crate rustc_serialize;
extern crate urlencoded;
mod params;

use rustc_serialize::json::Json;
use hyper::{Url,Client};
use iron::prelude::*;
use iron::status;
use iron::{Handler};
use iron::modifier::Modifier;
use std::fmt;
use std::fs::File;
use std::io;
use params::ImageTransformation;
use std::error::Error;

#[derive(Debug)]
struct InternalError {
    desc: String
}

impl InternalError {
    fn new(s: &str) -> InternalError {
        InternalError { desc: s.to_owned() }
    }
}

impl std::error::Error for InternalError {
    fn description(&self) -> &str {
        &self.desc
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Internal error: {}", self.desc)
    }
}

struct ImageHandler {
    client: Client
}

impl ImageHandler {
    fn new() -> Self {
        return ImageHandler { client: Client::new() };
    }

    fn fetch_image_from_content(&self, response: &mut hyper::client::Response, path: &str) -> IronResult<hyper::client::Response> {
        match Json::from_reader(response) {
            Ok(json) =>
                get_image_uri(json, path)
                .ok_or(IronError::new(io::Error::new(io::ErrorKind::NotFound, "No URI for image found"),
                                      status::NotFound))
                .and_then(|uri| self.fetch_image(&uri)),
            Err(e) => Err(IronError::new(e, status::InternalServerError))
        }
    }

    fn fetch_image<'a>(&self, uri: &str) -> IronResult<hyper::client::Response> {
        let url = itry!(Url::parse(uri), (status::InternalServerError,
                                          format!("Invalid image URI: {}", uri)));
        let host = match url.host_str() {
            Some(hostname) => hostname,
            None => return {
                let msg = format!("Image URI has no host: {}", uri);
                Err(IronError::new(InternalError::new(&msg), (status::InternalServerError,
                                                              msg)))
            }
        };
        let image_data_url = format!("http://localhost:8080/ace/file/{}/{}/{}", url.scheme(), host, url.path());
        let r = self.client.get(&image_data_url).send();
        match r {
            Ok(response) => {
                return Ok(response);
            },
            Err(e) => { let msg = format!("Couldn't fetch image: {}", 
                                                     e.description());
                        Err(IronError::new(e, (status::NotFound, msg)))
            }
        }
    }
}

#[derive(Debug)]
struct ContentImageInfo {
    pub path: String,
    pub content_id: String
}

impl ContentImageInfo {
    fn from_path(path: &[String]) -> Option<ContentImageInfo> {
        if path.len() < 2 {
            return None
        }
        return Some(ContentImageInfo { content_id: String::from(path[0].clone()), path: path[1..].join("/") })
    }
}

struct ImageReader {
    file: String
}

impl ImageReader {
    fn new(f: &str) -> ImageReader {
        ImageReader { file: f.to_owned() }
    }
}

impl Modifier<Response> for ImageReader {
    fn modify(self, res: &mut iron::response::Response) {
        File::open(&self.file).expect(&format!("Temp file not found: {}", &self.file)).modify(res)
    }
}

fn get_image_uri(json: Json, path: &str) -> Option<String> {
    return json.find_path(&["aspects","atex.Files","data","files", path, "fileUri"]).and_then(Json::as_string).map(str::to_owned)
}

fn transform_image(img: &mut io::Read, _: Option<ImageTransformation>) -> IronResult<Response> {
    let filename = "/tmp/img";
    let mut out = match File::create(filename) {
        Ok(f) => f,
        Err(x) => return Err(IronError::new(x, status::InternalServerError))
    };
    io::copy(img, &mut out)
        .map_err(|e| IronError::new(e, status::InternalServerError))
        .and(Ok(Response::with((status::Ok, ImageReader::new(filename)))))
}

impl Handler for ImageHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let transform = try!(ImageTransformation::from_request(req));
        let parsed = ContentImageInfo::from_path(&req.url.path[..]);
        match parsed {
            None => return Ok(Response::with(status::BadRequest)),
            Some(info) => {
                match self.client.get(&format!("{}/{}", "http://localhost:8080/ace/content/contentid", &info.content_id)).send() {
                    Ok(mut response) => {
                        let img = self.fetch_image_from_content(&mut response, &info.path);
                        match img {
                            Ok(mut img) => transform_image(&mut img, transform),
                            Err(e) => Err(e)
                        }
                    }
                    Err(_) => Ok(Response::with(status::NotFound))
                }
            }
        }
    }
}

fn main() {
    Iron::new(ImageHandler::new()).http("localhost:3000").unwrap();
}


#[cfg(test)]
mod tests {
use super::ContentImageInfo;

    #[test]
    fn image_simple_path() {
        let res = ContentImageInfo::from_path(&vec!["onecms:123".to_owned(), "apa.jpg".to_owned()]);
        match res {
            Some(info) => {
                assert_eq!("apa.jpg", info.path);
                assert_eq!("onecms:123", info.content_id);
            },
            None => panic!("Should get content image info")
        }
    }

    #[test]
    fn image_too_short_path() {
        let res = ContentImageInfo::from_path(&vec!["onecms:123".to_owned()]);
        match res {
            Some(_) => panic!("Should get none when path too short"),
            None => {}
        }
    }

    #[test]
    fn image_long_path() {
        let res = ContentImageInfo::from_path(&vec!["onecms:123".to_owned(), "apa".to_owned(), "bapa.jpg".to_owned()]);
        match res {
            Some(info) => {
                assert_eq!("onecms:123", info.content_id);
                assert_eq!("apa/bapa.jpg", info.path);
            },
            None => panic!("Should get content image info")
        }
    }

}
