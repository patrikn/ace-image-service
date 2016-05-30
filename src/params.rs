extern crate iron;
extern crate urlencoded;

use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedQuery;
use std::collections::HashMap;
use std::str::FromStr;
use std::fmt;

#[derive(Debug)]
pub struct ImageTransformation {
    pub width: u32,
    pub height: u32
}

impl fmt::Display for ImageTransformation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[width:{},height:{}]", self.width, self.height)
    }
}

impl ImageTransformation {
    pub fn from_request(req: &mut Request) -> IronResult<Option<ImageTransformation>> {
        match req.get_ref::<UrlEncodedQuery>() {
            Ok(ref params) => {
                let width = int_param(&params, &"w".to_owned());
                let height = int_param(&params, &"h".to_owned());
                if width > 0 && height > 0 {
                    Ok(Some(ImageTransformation { width: width as u32, height: height as u32 }))
                } else {
                    Ok(None)
                }
            },
            Err(e) => {
                Err(IronError::new(e, status::BadRequest))
            }
        }
    }
}

fn int_param(params: &HashMap<String, Vec<String>>, param: &String) -> i32 {
    params.get(param)
        .and_then(|vals| vals.first())
        .and_then(|x| FromStr::from_str(x).ok())
        .unwrap_or(-1)
}
