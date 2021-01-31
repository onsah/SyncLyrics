export CARGO_HOME=$1/target/cargo-home

cargo build --release -p sync_lyrics && cp $1/target/release/sync_lyrics $2