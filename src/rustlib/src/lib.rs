#[macro_use]
// extern crate lazy_static;
extern crate rustr;
extern crate rtsonlib;
extern crate rustson;
extern crate url;
extern crate bytes;

// extern crate futures;
// extern crate tokio;
extern crate hyper;
extern crate hyper_sync_rustls;

use hyper::Client;


use std::env;

pub mod export;

use url::Url;

use std::sync::mpsc;

use std::io::{Cursor, Write};

use rustr::*;
use rtsonlib::*;
use rtsonlib::ser::{RSerializer, Writer};

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
use hyper_sync_rustls::TlsClient;
use hyper::net::{SslClient, HttpConnector, HttpsConnector, Streaming};
use hyper::client::{ProxyConfig, RequestBuilder, Request, pool};
use hyper::header::{Headers, Connection, Host, ContentLength, Location};
use hyper::http::h1::Http11Protocol;
use hyper::http::Protocol;
use std::str::FromStr;


type TsonResult<T> = std::result::Result<T, TsonError>;

type RTsonResult<T> = std::result::Result<T, RTsonError>;

fn tercen_http_error(msg: &str) -> RError {
    RError::unknown("teRcenHttp -- ".to_string() + msg)
}


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
    let tls = hyper_sync_rustls::TlsClient::new();
    let client = match env::var("HTTP_PROXY") {
        Ok(mut proxy) => {
            // parse the proxy, message if it doesn't make sense
            let mut port = 80;
            if let Some(colon) = proxy.rfind(':') {
                port = proxy[colon + 1..].parse().unwrap_or_else(|e| {
                    panic!(
                        "HTTP_PROXY is malformed: {:?}, port parse error: {}",
                        proxy,
                        e
                    );
                });
                proxy.truncate(colon);
            }

            // connector here gets us to the proxy. tls then is used for https
            // connections via the proxy (tunnelled through the CONNECT method)
            let connector = HttpConnector::default();
            let proxy_config = ProxyConfig::new("http", proxy, port, connector, tls);
            Client::with_proxy_config(proxy_config)
        }
        _ => {
            let connector = HttpsConnector::new(tls);
            Client::with_connector(connector)
        }
    };

    // let mut m_headers = Headers::new();
    // headers.iter()
    //     .for_each(|e| m_headers.set_raw(e.0.to_string(),
    //                                              vec![ e.1.as_bytes().to_vec()] ));

    let protocol = if url.scheme() == "https" {
        let tls = hyper_sync_rustls::TlsClient::new();
        Http11Protocol::with_connector(HttpsConnector::new(tls))
    } else {
        Http11Protocol::with_connector(hyper::client::pool::Pool::new(pool::Config::default()))
    };


    let mut req = {
        let (host, port) = get_host_and_port(&url).unwrap();
        let message = protocol.new_message(host, port, url.scheme()).unwrap();

        // let mut message = try!(client.protocol.new_message(&host, port, url.scheme()));
        // if url.scheme() == "http" && client.proxy.is_some() {
        //     message.set_proxied(true);
        // }

        let mut h = Headers::new();
        h.set(Host {
            hostname: host.to_owned(),
            port: Some(port),
        });
        for e in headers.clone().into_iter() {
            h.set_raw(e.0, vec![e.1.as_bytes().to_vec()])
        }

        let headers = h;
        Request::with_headers_and_message(Method::from_str(verb.as_str()).unwrap(),
                                          url, headers, message)
    };

    // try!(req.set_write_timeout(client.write_timeout));
    // try!(req.set_read_timeout(client.read_timeout));
    // let can_have_body = match method {
    //     Method::Get | Method::Head => false,
    //     _ => true
    // };

    // let mut body = if can_have_body {
    //     body
    // } else {
    //     None
    // };
    //
    // match (can_have_body, body.as_ref()) {
    //     (true, Some(body)) => match body.size() {
    //         Some(size) => req.headers_mut().set(ContentLength(size)),
    //         None => (), // chunked, Request will add it automatically
    //     },
    //     (true, None) => req.headers_mut().set(ContentLength(0)),
    //     _ => () // neither
    // }

    type Result<T> = std::result::Result<T, TsonError>;

    let mut streaming = SenderWriter::new(req.start().unwrap());
    body_writer.write(&mut streaming)?;

    let mut res = streaming.sender.send().unwrap();

    ResponseReader::new(response_type.into()).read(&mut res)


    // if !res.status.is_redirection() {
    //     return Ok(res)
    // }
    // debug!("redirect code {:?} for {}", res.status, url);
    //
    // let loc = {
    //     // punching borrowck here
    //     let loc = match res.headers.get::<Location>() {
    //         Some(&Location(ref loc)) => {
    //             Some(url.join(loc))
    //         }
    //         None => {
    //             debug!("no Location header");
    //             // could be 304 Not Modified?
    //             None
    //         }
    //     };
    //     match loc {
    //         Some(r) => r,
    //         None => return Ok(res)
    //     }
    // };
    // url = match loc {
    //     Ok(u) => u,
    //     Err(e) => {
    //         debug!("Location header had invalid URI: {:?}", e);
    //         return Ok(res);
    //     }
    // };
    // match client.redirect_policy {
    //     // separate branches because they can't be one
    //     RedirectPolicy::FollowAll => (), //continue
    //     RedirectPolicy::FollowIf(cond) if cond(&url) => (), //continue
    //     _ => return Ok(res),
    // }

    // body_writer.write(&req);

    // let builder = client
    //     .request(verb.parse().unwrap(),
    //              &url.to_string())
    //     .headers(m_headers).send()
    //     ;


    // let mut res = client
    //     .get(&url.to_string())
    //     .header(Connection::close())
    //     .send()
    //     .unwrap();
    //
    // println!("Response: {}", res.status);
    // println!("Headers:\n{}", res.headers);
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

