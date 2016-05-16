extern crate iron;

use iron::prelude::*;
use iron::status;
use iron::{Handler};


struct ImageHandler {
}

impl ImageHandler {
    fn new() -> Self {
        return ImageHandler {};
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

impl Handler for ImageHandler {
    fn handle(&self, req: &mut Request) -> IronResult<Response> {
        let parsed = ContentImageInfo::from_path(&req.url.path[..]);
        match parsed {
            None => return Ok(Response::with(status::BadRequest)),
            Some(info) => return Ok(Response::with((status::Ok, info.content_id + "/" + info.path.as_str())))
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
