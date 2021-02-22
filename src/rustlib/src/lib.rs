extern crate rustr;
extern crate rtsonlib;
extern crate rustson;
extern crate url;
extern crate bytes;
extern crate hyper;
extern crate hyper_sync_rustls;

pub mod export;
use url::Url;
use std::io::{Cursor, Write};
use rustr::*;
use rtsonlib::*;
use rtsonlib::ser::{RSerializer};
use rustson::TsonError;
use rustson::Value::*;
use std::collections::HashMap;

mod ser;
mod deser;
mod multi;

use deser::*;
use ser::*;
use multi::*;
use hyper::method::Method;

use hyper::net::{HttpsConnector, Streaming};
use hyper::client::{Request, pool};
use hyper::header::{Headers,  Host};
use hyper::http::h1::Http11Protocol;
use hyper::http::Protocol;
use std::str::FromStr;


type TsonResult<T> = std::result::Result<T, TsonError>;

type RTsonResult<T> = std::result::Result<T, RTsonError>;

// #[rustr_export]
pub fn to_tson(object: SEXP) -> RResult<RawVec> {
    rtsonlib::to_tson(object)
}

// #[rustr_export]
pub fn from_tson(rbytes: RawVec) -> RResult<SEXP> {
    rtsonlib::from_tson(rbytes)
}

// #[rustr_export]
pub fn to_json(object: SEXP) -> RResult<String> {
    rtsonlib::to_json(object)
}

// #[rustr_export]
pub fn from_json(data: String) -> RResult<SEXP> {
    rtsonlib::from_json(&data)
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
    let mut uri = Url::parse(&url).map_err(|e| RError::other(e))?;
    for (key, value) in query.iter() {
        uri.query_pairs_mut().append_pair(key, value);
    }
    do_verb_url_r(verb, hheaders, uri, multipart, response_type)
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

            let mut uri = Url::parse(&url).map_err(|e| RError::other(e))?;
            for (key, value) in query.iter() {
                uri.query_pairs_mut().append_pair(key, value);
            }
            do_verb_url_r(verb, hheaders, uri, json, response_type)
        }
        "application/tson" => {
            hheaders.insert("content-type".to_string(), "application/tson".to_string());

            let mut uri = Url::parse(&url).map_err(|e| RError::other(e))?;
            for (key, value) in query.iter() {
                uri.query_pairs_mut().append_pair(key, value);
            }
            do_verb_url_r(verb, hheaders, uri, TsonBodyWriter::new(body), response_type)
        }
        _ => {
            hheaders.insert("content-type".to_string(), "application/octet-stream".to_string());

            let mut uri = Url::parse(&url).map_err(|e| RError::other(e))?;
            for (key, value) in query.iter() {
                uri.query_pairs_mut().append_pair(key, value);
            }
            do_verb_url_r(verb, hheaders, uri, TsonBodyWriter::new(body), response_type)
        }
    }
}


// #[rustr_export]
pub fn do_verb(verb: String,
               headers: HashMap<String, String>,
               url: String,
               query: HashMap<String, String>,
               body: RawVec,
               response_type: String) -> RResult<SEXP> {
    let mut uri = Url::parse(&url).map_err(|e| RError::other(e))?;
    for (key, value) in query.iter() {
        uri.query_pairs_mut().append_pair(key, value);
    }

    do_verb_url_r(verb, headers, uri, body, response_type)
}

fn get_host_and_port(url: &Url) -> Result<(&str, u16), &str> {
    let host = match url.host_str() {
        Some(host) => host,
        None => return Err("Error::Uri(UrlError::EmptyHost)"),
    };
    // trace!("host={:?}", host);
    let port = match url.port_or_known_default() {
        Some(port) => port,
        None => return Err("Error::Uri(UrlError::InvalidPort)"),
    };
    // trace!("port={:?}", port);
    Ok((host, port))
}

pub fn do_verb_url_r<T>(verb: String,
                        headers: HashMap<String, String>,
                        url: Url,
                        body_writer: T,
                        response_type: String) -> RResult<SEXP> where T: BodyWriter {

    let protocol = if url.scheme() == "https" {
        let tls = hyper_sync_rustls::TlsClient::new();
        Http11Protocol::with_connector(HttpsConnector::new(tls))
    } else {
        Http11Protocol::with_connector(hyper::client::pool::Pool::new(pool::Config::default()))
    };

    let req = {
        let (host, port) = get_host_and_port(&url).map_err(|e| RError::unknown(e.to_string()))?;
        let message = protocol.new_message(host, port, url.scheme()).map_err(|e| RError::unknown(e.to_string()))?;

        let mut h = Headers::new();
        h.set(Host {
            hostname: host.to_owned(),
            port: Some(port),
        });
        for e in headers.clone().into_iter() {
            h.set_raw(e.0, vec![e.1.as_bytes().to_vec()])
        }

        let headers = h;
        Request::with_headers_and_message(Method::from_str(
            verb.as_str()).map_err(|e| RError::unknown(e.to_string()))?,
                                          url, headers, message)
    };

     let mut streaming = SenderWriter::new(req.start()
        .map_err(|e| RError::unknown(e.to_string()))?);

    if &verb == "GET" || &verb == "HEAD" {

    } else {

        body_writer.write(&mut streaming)?;
         streaming.close().map_err(|e| RError::unknown(e.to_string()))?;
     }

    let mut res = streaming.sender.send()
        .map_err(|e| RError::unknown(e.to_string()))?;



    ResponseReader::new(response_type.into()).read(&mut res)
}

// pub fn do_verb_url_r<T>(verb: String,
//                         headers: HashMap<String, String>,
//                         url: Url,
//                         body_writer: T,
//                         response_type: String) -> RResult<SEXP> where T: BodyWriter {
//     match *CLIENTR {
//         Ok(ref c) => {
//             let client: &Client<HttpsConnector<HttpConnector>, Body> = c;
//
//             let _runtime = tokio_global::multi::init();
//
//             let (request, sender) = build_request(verb, headers, url)?;
//
//             let (send, recv) = mpsc::channel::<Result<BodyItem, hyper::error::Error>>();
//
//             let send_done = send.clone();
//             let send_clone_err = send.clone();
//
//             client.request(request).and_then(move |response| {
//                 let (parts, body) = response.into_parts();
//
//                 send.send(Ok(BodyItem::H(parts))).expect("send parts");
//
//                 body.for_each(move |chunk| {
//                     send.send(Ok(BodyItem::C(chunk))).expect("send chunck");
//                     Ok(())
//                 })
//             }).and_then(move |_| {
//                 // channel can be closed because the ResponseReader did not read to the end ...
//                 match send_done.send(Ok(BodyItem::Done)) {
//                     Ok(_) => Ok(()),
//                     Err(_) => {
//                         Ok(())
//                     },
//                 }
//             }).map_err(move |err| {
//                 send_clone_err.send(Err(err)).expect("send error");
//             }).spawn();
//
//             // Send the body using sender
//
//             futures::lazy(|| {
//                 let mut writer = SenderWriter::new(sender);
//                 body_writer.write(&mut writer)?;
//                 writer.close().map_err(|e| RError::unknown(e.to_string()))
//             }).wait()?;
//
//             // let mut writer = SenderWriter::new(sender);
//             // body_writer.write(&mut writer)?;
//             // writer.close().map_err(|e| RError::unknown(e.to_string()))?;
//
//             ResponseReader::new(response_type.into()).read(&recv)
//
//         }
//         Err(ref e) => {
//             Err(tercen_http_error(&e.to_string()))
//         }
//     }
// }


// #[cfg(test)]
// mod tests {
//     #[test]
//     fn test_poll_fn() {
//         println!("hey test_poll_fn");
//
//         use futures::Future;
//         use futures::future::poll_fn;
//         use futures::{Async, Poll};
//
//         let mut counter = 10;
//
//         let read_future = poll_fn(move || -> Poll<String, std::io::Error> {
//             if counter < 0 {
//                 return Ok(Async::Ready("Hello, World!".into()));
//             }
//             counter -= 1;
//             Ok(Async::NotReady)
//         });
//
//
//         let r = read_future.wait().unwrap();
//
//         println!("result {}", r);
//     }
// }

