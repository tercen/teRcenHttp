R http client package

# Install 

```R
devtools::install_github("tercen/teRcenHttp", ref = "1.0.5", args="--no-multiarch")
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
 
 
```R
 
library(teRcenHttp)

GET("https://dev.tercen.com")
GET("https://dev.tercen.com", response_type="application/octet-stream")

toJSON(list(hey="42", you=tson.scalar(42.0)))
toJSON(list(hey="42", you=42.0))

toTSON(list(hey="42", you=tson.scalar(42.0)))

fromJSON(toJSON(list(hey="42", you=tson.scalar(42.0))))
fromJSON(toJSON(list(hey=tson.scalar(42), you=tson.scalar(42.0))))

fromTSON(toTSON(list(hey="42", you=tson.scalar(42.0))))
fromTSON(toTSON(list(hey="42", you=42.0)))

fromJSON('{"hey":"42", "you":42.0}')
fromJSON('{"hey":"42", "you":[42.0]}')
```
