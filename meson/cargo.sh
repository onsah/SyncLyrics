
export CARGO_HOME=$1/target/cargo-home

cargo build -p sync_lyrics && cp $1/target/debug/sync_lyrics $2