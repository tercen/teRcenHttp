#[macro_use]
extern crate rustr;
extern crate reqwest;
extern crate rtsonlib;
extern crate rustson;
extern crate url;
extern crate bytes;

pub mod export;

use bytes::{BufMut};
use std::str::FromStr;
use std::io::Cursor;
use std::time::Duration;

pub use reqwest::header::*;
pub use reqwest::*;
pub use rustr::*;
pub use rtsonlib::*;
pub use rustson::*;

use std::collections::HashMap;
use rtsonlib::Value::*;

struct Part {
    headers: HashMap<String, String>,
    content: Vec<u8>,
}

impl Part {
    fn from_value(part_value: Value) -> RResult<Part> {
        match part_value {
            MAP(mut map) => {
                let headers_value = map.remove("headers")
                    .ok_or(reqwestr_error("headers is required"))?;
                let content_value = map.remove("content")
                    .ok_or(reqwestr_error("content is required"))?;

                match headers_value {
                    MAP(headers_val) => {
                        let mut headers: HashMap<String, String> = HashMap::new();
                        for (k, value_value) in headers_val {
                            match value_value {
                                STR(value) => {
                                    headers.insert(k.to_string(), value.to_string());
                                }
                                _ => return Err(reqwestr_error("header values must be string")),
                            }
                        }

                        let content_type = headers.get("content-type")
                            .ok_or(reqwestr_error("headers.content-type is required"))?;

                        match content_type.as_ref() {
                            "application/octet-stream" => {
                                match content_value {
                                    LSTU8(content) => Ok(Part{ headers: headers.clone(), content: content }),
                                    _ => Err(reqwestr_error("for content type application/octet-stream content must be a raw vector")),
                                }
                            }
                            "application/tson" => {
                                match encode(&content_value) {
                                    Ok(content) => {
                                        Ok(Part { headers: headers.clone(), content: content })
                                    }
                                    Err(ref e) => Err(reqwestr_error(e)),
                                }
                            }
                            "application/json" => {
                                let json = rtsonlib::encode_json(&content_value).map_err(|e| reqwestr_error(&e))?;
                                Ok(Part { headers: headers.clone(), content: json.into_bytes() })
                            }
                            _ => Err(reqwestr_error("unknown content-type")),
                        }
                    }
                    _ => Err(reqwestr_error("headers must be a map")),
                }
            }
            _ => Err(reqwestr_error("body must be a list of map"))
        }
    }
}

fn reqwestr_error(msg: &str) -> RError {
    RError::unknown("reqwestr -- ".to_string() + msg)
}

struct MultiPart {
    frontier: String,
    parts: Vec<Part>
}

impl MultiPart {
    fn from_r(object: SEXP) -> RResult<MultiPart> {
        let value = r_to_value(object)?;
        match value {
            LST(list) => {
                let mut parts = Vec::new();

                for part in list {
                    parts.push(Part::from_value(part)?);
                }

                Ok(MultiPart { frontier: "ab63a1363ab349aa8627be56b0479de2".to_string(), parts: parts })
            }
            _ => Err(reqwestr_error("body must be a list")),
        }
    }

    fn as_bytes(&self) -> Vec<u8> {

        let mut len : usize = 0;
        for part in &self.parts {
            len += 2;
            len += &self.frontier.len();
            len += 2;

            for (k,v) in &part.headers {
                len += k.len();
                len += v.len();
                len += 4;
            }

            len += 2;
            len += part.content.len();
            len += 2;
        }

        len += 2;
        len += &self.frontier.len();
        len += 2;
        len += 2;

        let mut bytes = Vec::with_capacity(len + 1000);

        for part in &self.parts {
            bytes.put("--");
            bytes.put(&self.frontier);
            bytes.put_u8(13);
            bytes.put_u8(10);

            for (k,v) in &part.headers {
                bytes.put(k);
                bytes.put(": ");
                bytes.put(v);
                bytes.put_u8(13);
                bytes.put_u8(10);
            }

            bytes.put_u8(13);
            bytes.put_u8(10);

            bytes.put(&part.content);

            bytes.put_u8(13);
            bytes.put_u8(10);
        }

        bytes.put("--");
        bytes.put(&self.frontier);
        bytes.put("--");
        bytes.put_u8(13);
        bytes.put_u8(10);

        bytes
    }
}


// #[rustr_export]
pub fn do_verb_multi_part_r(verb: String,
                            headers: HashMap<String, String>,
                            url: String,
                            query: HashMap<String, String>,
                            body: SEXP,
                            response_type: String) -> RResult<SEXP> {
    let multipart = MultiPart::from_r(body)?;
    let mut hheaders = headers.clone();
    hheaders.insert("content-type".to_string(),
                    "multipart/mixed; boundary=".to_string() + &multipart.frontier);
    do_verb(verb, hheaders, url, query, multipart.as_bytes(), response_type)
}

// #[rustr_export]
pub fn do_verb_r(verb: String,
                 headers: HashMap<String, String>,
                 url: String,
                 query: HashMap<String, String>,
                 body: SEXP,
                 content_type: String,
                 response_type: String) -> RResult<SEXP> {
    let mut hheaders = headers.clone();

    match content_type.as_ref() {
        "application/json" => {
            hheaders.insert("content-type".to_string(), "application/json".to_string());
            let json = rtsonlib::to_json(body)?;
            do_verb(verb, headers, url, query, json.into_bytes(), response_type)
        }
        "application/tson" => {
            hheaders.insert("content-type".to_string(), "application/tson".to_string());
            let value = r_to_value(body)?;
            match encode(&value) {
                Ok(buf) => {
                    do_verb(verb, headers, url, query, buf, response_type)
                }
                Err(ref e) => Err(reqwestr_error(e)),
            }
        }
        _ => {
            let value = r_to_value(body)?;
            match encode(&value) {
                Ok(buf) => {
                    do_verb(verb, headers, url, query, buf, response_type)
                }
                Err(ref e) => Err(reqwestr_error(e)),
            }
        }
    }
}

// #[rustr_export]
pub fn do_verb(verb: String,
               headers: HashMap<String, String>,
               url: String,
               query: HashMap<String, String>,
               body: Vec<u8>,
               response_type: String) -> RResult<SEXP> {
    match Url::parse(&url) {
        Ok(mut uri) => {
            for (key, value) in query.iter() {
                uri.query_pairs_mut().append_pair(key, value);
            }

            do_verb_url(verb, headers, uri, body, response_type)
        }
        Err(ref e) => Err(reqwestr_error(&e.to_string())),
    }
}

pub fn do_verb_url(verb: String,
                   headers: HashMap<String, String>,
                   url: Url,
                   body: Vec<u8>,
                   response_type: String) -> RResult<SEXP> {
//    let client = Client::new();

    let client = reqwest::Client::builder()
//        .gzip(true)
        .timeout(Duration::from_secs(36000))
        .build().map_err(|e| reqwestr_error(&e.to_string()))?;


    let mut request_builder = client.request(Method::from_str(&verb).unwrap(), url);
    for (key, value) in headers.iter() {
        request_builder = request_builder.header(key.as_str(), value.as_str());
    }

    if !body.is_empty() {
        request_builder = request_builder.body(body);
    }

    match request_builder.send() {
        Err(_error) => rraise(_error),
        Ok(mut response) => {
            let mut buf: Vec<Rbyte> = vec![];
            match response.copy_to(&mut buf) {
                Err(_error) => rraise(_error),
                Ok(_) => {
                    let mut names = CharVec::alloc(3);
                    let mut values = RList::alloc(3);

                    names.set(0, "status")?;
                    values.set(0, response.status().as_u16().intor()?)?;
                    names.set(1, "headers")?;

                    let mut h: Vec<String> = vec![];

                    for (key, value) in response.headers().iter() {
                        h.push(key.as_str().to_string());
                        match value.to_str() {
                            Err(error) => {
                                return rraise(error);
                            }
                            Ok(v) => {
                                h.push(v.to_string());
                            }
                        }
                    }

                    values.set(1, h.intor()?)?;
                    names.set(2, "content")?;

                    let mut resp_type: &str = &response_type;

                    match response_type.as_ref() {
                        "default" => {
                            if let Some(content_type) = response.headers().get("content-type") {
                                resp_type = content_type.to_str().unwrap();
                            }
                        }
                        _ => {}
                    }

                    match resp_type {
                        "application/tson" => {
                            match decode(Cursor::new(&buf)) {
                                Ok(object) => {
                                    values.set(2, (value_to_r(&object)?).intor()?)?;
                                }
                                Err(ref e) => {
                                    return Err(reqwestr_error(e));
                                }
                            }
                        }
                        "application/json" => {
                            match decode_json(&buf) {
                                Ok(object) => {
                                    values.set(2, (value_to_r(&object)?).intor()?)?;
                                }
                                Err(ref e) => {
                                    return Err(reqwestr_error(e));
                                }
                            }
                        }
                        "application/octet-stream" => {
                            let mut raw_vec = RawVec::alloc(buf.len());

                            unsafe {
                                for i in 0..buf.len() {
                                    raw_vec.uset(i, buf[i]);
                                }
                            }

                            values.set(2, raw_vec.intor()?)?;
                        }
                        "text/html" => {
                            unsafe {
                                let utf8str = String::from_utf8_unchecked(buf);
                                values.set(2, utf8str.intor()?)?;
                            }
                        }
                        _ => {
                            let mut raw_vec = RawVec::alloc(buf.len());

                            unsafe {
                                for i in 0..buf.len() {
                                    raw_vec.uset(i, buf[i]);
                                }
                            }

                            values.set(2, raw_vec.intor()?)?;
                        }
                    }

                    unsafe {
                        Rf_setAttrib(values.s(), R_NamesSymbol, names.s());
                    }

                    values.intor()
                }
            }
        }
    }
}
