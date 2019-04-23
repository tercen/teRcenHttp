TSON_KIND = "kind"
TSON_SCALAR = -1

#' Make a tson map
#'
#' Required to generate empty map.
#'
#' @param object A vector or list
#' @return A tson map
tson.map = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("tsonmap", class(object))
  return (object)
}

#' Make a tson float32 vector
#'
#'
#' @param object A vector or list
#' @return A tson float32 vector
tson.float32.vec = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("float32", class(object))
  return (object)
}

#' Make a tson int8 vector
#'
#'
#' @param object A vector or list
#' @return A tson int8 vector
tson.int8.vec = function(object){
  if (is.null(object)) return(NULL)
  object = as.integer(object)
  class(object) <- c("int8", class(object))
  return (object)
}

#' Make a tson int16 vector
#'
#'
#' @param object A vector or list
#' @return A tson int16 vector
tson.int16.vec = function(object){
  if (is.null(object)) return(NULL)
  object = as.integer(object)
  class(object) <- c("int16", class(object))
  return (object)
}

#' Make a tson int16 vector
#'
#'
#' @param object A vector or list
#' @return A tson int64 vector
tson.int64.vec = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("int64", class(object))
  return (object)
}

#' Make a tson uint8 vector
#'
#'
#' @param object A vector or list
#' @return A tson uint8 vector

tson.uint8.vec = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("uint8", class(object))
  return (object)
}
#' Make a tson uint16 vector
#'
#'
#' @param object A vector or list
#' @return A tson uint16 vector
tson.uint16.vec = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("uint16", class(object))
  return (object)
}

#' Make a tson uint32 vector
#'
#'
#' @param object A vector or list
#' @return A tson uint32 vector
tson.uint32.vec = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("uint32", class(object))
  return (object)
}

#' Make a tson uint64 vector
#'
#'
#' @param object A vector or list
#' @return A tson uint64 vector
tson.uint64.vec = function(object){
  if (is.null(object)) return(NULL)
  class(object) <- c("uint64", class(object))
  return (object)
}

is.namedlist = function(obj) {
  return (!is.null(names(obj)))
}

#' Make a tson scalar (ie: singleton)
#'
#' @param object A vector or list
#' @return A tson scalar
tson.scalar = function(obj){
  if (is.null(obj)) return(NULL)
  # Lists can never be a scalar (this can arise if a dataframe contains a column
  # with lists)
  if(length(dim(obj)) > 1){
    if(!identical(nrow(obj), 1L)){
      warning("Tried to use as.scalar on an array or dataframe with ", nrow(obj), " rows.", call.=FALSE)
      return(obj)
    }
  } else if(!identical(length(obj), 1L)) {
    warning("Tried to use as.scalar on an object of length ", length(obj), call.=FALSE)
    return(obj)
  } else if(is.namedlist(obj)){
    warning("Tried to use as.scalar on a named list.", call.=FALSE)
    return(obj)
  }

  class(obj) <- c("scalar", class(obj))
  return(obj)
}

#' Make a tson integer
#'
#'
#' @param object A vector or list
#' @return A tson integer
tson.int = function(object){
  if (is.null(object)) return(NULL)
  return (tson.scalar(as.integer(object)))
}

#' Make a tson double
#'
#'
#' @param object A vector or list
#' @return A tson double
tson.double = function(object){
  if (is.null(object)) return(NULL)
  return (tson.scalar(as.double(object)))
}

#' Make a tson character
#'
#' @param object A vector or list
#' @return A tson character
tson.character = function(object){
  if (is.null(object)) return(NULL)
  return (tson.scalar(as.character(object)))
}





