dev:
    cargo watch -w src -x clippy -s "lsof -ti tcp:8899 | xargs kill -9 && cargo run"