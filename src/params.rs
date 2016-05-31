extern crate iron;
extern crate urlencoded;

use iron::prelude::*;
use iron::status;
use urlencoded::{UrlEncodedQuery, UrlDecodingError};
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
        from_params(req.get_ref::<UrlEncodedQuery>())
    }
}

pub fn from_params(params: Result<&HashMap<String, Vec<String>>, UrlDecodingError>) -> IronResult<Option<ImageTransformation>> {
    match params {
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

fn int_param(params: &HashMap<String, Vec<String>>, param: &String) -> i32 {
    params.get(param)
        .and_then(|vals| vals.first())
        .and_then(|x| FromStr::from_str(x).ok())
        .unwrap_or(-1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn no_params() {
        let req: HashMap<String, Vec<String>> = HashMap::new();
        assert_eq!(true, from_params(Ok(&req)).unwrap().is_none());
    }
}
