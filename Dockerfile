FROM rust:1.76-slim-bookworm

WORKDIR /usr/src

COPY ./ ./

RUN apt update
RUN apt install -y python3-pip ffmpeg
RUN pip3 install whisper-timestamped --break-system-packages

RUN cargo build --release

CMD ["./target/release/app-server"]
