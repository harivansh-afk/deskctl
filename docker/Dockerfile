FROM rust:1.94-bullseye

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY assets ./assets
COPY src ./src

RUN cargo build --release --locked

CMD ["bash", "-lc", "mkdir -p /out && cp target/release/deskctl /out/deskctl-linux-x86_64 && chmod 755 /out/deskctl-linux-x86_64"]
