R http client package

# Build 

```R
devtools::build()
devtools::install()
```

# Install 

```R
devtools::install_github("tercen/teRcenHttp", ref = "1.0.6", args="--no-multiarch")
devtools::install_github("tercen/teRcenHttp", args="--no-multiarch")
```

# Build rust

```bash
R -e "rustinr::rustrize()"
cd src/rustlib
cargo build
```

# compilation

```bash
cd src/rustlib
cross build --release --target x86_64-pc-windows-gnu
```
 
# Example 

```R
teRcenHttp::GET("https://tercen.com")

teRcenHttp::toTSON(c("paste_code=printf();"))
teRcenHttp::toTSON("fgfg")
teRcenHttp::toTSON(1)

bytes = teRcenHttp::toTSON(list(name="alex", age=teRcenHttp::tson.scalar(42)))
teRcenHttp::fromTSON(bytes)
teRcenHttp::toJSON(list(name="alex", age=teRcenHttp::tson.scalar(42)))
teRcenHttp::toJSON(list(name="alex", age=(42)))

teRcenHttp::POST("http://127.0.0.1:4040", body="hello")
teRcenHttp::POST("http://127.0.0.1:4040", body="hello", content_type="application/json")
teRcenHttp::POST("http://127.0.0.1:4040", body="hello", content_type="application/octet-stream")
teRcenHttp::POST("http://127.0.0.1:4040", 
    body=list(name=teRcenHttp::tson.scalar("alex"), age=teRcenHttp::tson.scalar(42)))
teRcenHttp::POST("http://127.0.0.1:4040", 
    body=list(name=teRcenHttp::tson.scalar("alex"), age=teRcenHttp::tson.scalar(42)), content_type="application/json")
teRcenHttp::POST("http://127.0.0.1:4040", body=teRcenHttp::POST)
teRcenHttp::POST("http://127.0.0.1:4040", body=NaN)
teRcenHttp::POST("http://127.0.0.1:4040", body=NaN, content_type="application/json")

teRcenHttp::POST("http://127.0.0.1:4040", body=seq(0,100))
teRcenHttp::POST("http://127.0.0.1:4040", body=seq(0,100000000))
httr::POST("http://127.0.0.1:4040", body=seq(0,100000),  encode="json")
                
rbenchmark::benchmark("teRcenHttp" = {
            res = teRcenHttp::POST("http://127.0.0.1:4040", 
                body=list(name=teRcenHttp::tson.scalar("alex"), age=teRcenHttp::tson.scalar(42), list=seq(0,100000)))
            # print(res)
          },
          "httr" = {
            res = httr::POST("http://127.0.0.1:4040", 
                            body=list(name=teRcenHttp::tson.scalar("alex"), age=teRcenHttp::tson.scalar(42), list=seq(0,100000)), encode="json")
            # print(res)
          },
          replications = 10,
          columns = c("test", "replications", "elapsed",
                      "relative", "user.self", "sys.self"))
                      
                      
rbenchmark::benchmark("teRcenHttp" = {
            res = teRcenHttp::GET("http://tercen.com")
            # print(res)
          },
          "httr" = {
            res = httr::GET("http://tercen.com")
            # print(res)
          },
          replications = 10,
          columns = c("test", "replications", "elapsed",
                      "relative", "user.self", "sys.self"))
```
