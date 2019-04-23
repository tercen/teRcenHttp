R http client package

# Install 

```R
devtools::install_github("tercen/teRcenHttp", ref = "1.0.2", upgrade_dependencies = FALSE, args="--no-multiarch")
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

teRcenHttp::GET("https://dev.tercen.com")
 
```
