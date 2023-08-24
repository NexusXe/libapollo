Push-Location C:\Users\andas\Documents\Rust_projects\apollo\
$Env:RUST_BACKTRACE=1
cargo rustc -- -C target-cpu=native -C opt-level=s -C codegen-units=1 --emit asm --crate-type=lib -C debuginfo=0 -C debug-assertions=false -C lto=true -C embed-bitcode=true
Pop-Location
