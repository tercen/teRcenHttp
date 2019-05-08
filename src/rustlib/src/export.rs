// Generated by using rustinr::rustrize() -> do not edit by hand

use super::*;

#[no_mangle]
pub extern "C" fn rustr_to_tson(object : SEXP)->SEXP{

 let object_ : SEXP = unwrapr!( SEXP::rnew(object) );
 let res  = unwrapr!( to_tson(object_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

#[no_mangle]
pub extern "C" fn rustr_from_tson(rbytes : SEXP)->SEXP{

 let rbytes_ : RawVec = unwrapr!( RawVec::rnew(rbytes) );
 let res  = unwrapr!( from_tson(rbytes_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

#[no_mangle]
pub extern "C" fn rustr_to_json(object : SEXP)->SEXP{

 let object_ : SEXP = unwrapr!( SEXP::rnew(object) );
 let res  = unwrapr!( to_json(object_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

#[no_mangle]
pub extern "C" fn rustr_from_json(data : SEXP)->SEXP{

 let data_ : String = unwrapr!( String::rnew(data) );
 let res  = unwrapr!( from_json(data_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

#[no_mangle]
pub extern "C" fn rustr_do_verb_multi_part_r(verb : SEXP, headers : SEXP, url : SEXP, query : SEXP, body : SEXP, response_type : SEXP)->SEXP{

 let verb_ : String = unwrapr!( String::rnew(verb) );

let headers_ : HashMap<String, String> = unwrapr!( HashMap::rnew(headers) );

let url_ : String = unwrapr!( String::rnew(url) );

let query_ : HashMap<String, String> = unwrapr!( HashMap::rnew(query) );

let body_ : SEXP = unwrapr!( SEXP::rnew(body) );

let response_type_ : String = unwrapr!( String::rnew(response_type) );
 let res  = unwrapr!( do_verb_multi_part_r(verb_,headers_,url_,query_,body_,response_type_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

#[no_mangle]
pub extern "C" fn rustr_do_verb_r(verb : SEXP, headers : SEXP, url : SEXP, query : SEXP, body : SEXP, content_type : SEXP, response_type : SEXP)->SEXP{

 let verb_ : String = unwrapr!( String::rnew(verb) );

let headers_ : HashMap<String, String> = unwrapr!( HashMap::rnew(headers) );

let url_ : String = unwrapr!( String::rnew(url) );

let query_ : HashMap<String, String> = unwrapr!( HashMap::rnew(query) );

let body_ : SEXP = unwrapr!( SEXP::rnew(body) );

let content_type_ : String = unwrapr!( String::rnew(content_type) );

let response_type_ : String = unwrapr!( String::rnew(response_type) );
 let res  = unwrapr!( do_verb_r(verb_,headers_,url_,query_,body_,content_type_,response_type_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

#[no_mangle]
pub extern "C" fn rustr_do_verb(verb : SEXP, headers : SEXP, url : SEXP, query : SEXP, body : SEXP, response_type : SEXP)->SEXP{

 let verb_ : String = unwrapr!( String::rnew(verb) );

let headers_ : HashMap<String, String> = unwrapr!( HashMap::rnew(headers) );

let url_ : String = unwrapr!( String::rnew(url) );

let query_ : HashMap<String, String> = unwrapr!( HashMap::rnew(query) );

let body_ : RawVec = unwrapr!( RawVec::rnew(body) );

let response_type_ : String = unwrapr!( String::rnew(response_type) );
 let res  = unwrapr!( do_verb(verb_,headers_,url_,query_,body_,response_type_));

 let res_sexp : SEXP = unwrapr!(res.intor());

 return res_sexp;
}

