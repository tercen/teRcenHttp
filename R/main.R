MULTIPART = function(url,
                     headers=structure(list(),
                                       names=character(0)),
                     query=structure(list(), names=character(0)),
                     body=NULL,
                     response_type="default"){
  do_verb_multi_part_r("POST",headers,url,query,body,response_type)
}

GET = function(url,
               headers=structure(list(),names=character(0)),
               query=structure(list(), names=character(0)),
               response_type="default") {
  do_verb_r("GET",headers,url,query,double(),"application/tson",response_type)
}

HEAD = function(url,
                headers=structure(list(),names=character(0)),
                query=structure(list(), names=character(0)),
                response_type="default") {
  do_verb_r("HEAD",headers,url,query,raw(),"application/octet-stream",response_type)
}

POST = function(url,
                headers=structure(list(),
                                  names=character(0)),
                query=structure(list(), names=character(0)),
                body=NULL,
                content_type="application/tson",
                response_type="default") {
  do_verb_r("POST",headers,url,query,body,content_type,response_type)
}

PUT = function(url,
               headers=structure(list(),
                                 names=character(0)),
               query=structure(list(), names=character(0)),
               body=NULL,
               content_type="application/tson",
               response_type="default") {
  do_verb_r("PUT",headers,url,query,body,content_type,response_type)
}



DELETE = function(url,
                  headers=structure(list(),
                                    names=character(0)),
                  query=structure(list(), names=character(0)),
                  body=NULL,
                  content_type="application/tson",
                  response_type="default") {
  do_verb_r("DELETE",headers,url,query,body,content_type,response_type)
}

fromJSON <- function(object){
  return(from_json(object))
}

toJSON <- function(object) {
  return(to_json(object))
}

fromTSON <- function(bytes){
  return(from_tson(bytes))
}

toTSON <- function(object) {
  return(to_tson(object))
}
