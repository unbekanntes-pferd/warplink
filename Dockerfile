# build
FROM rust:latest as builder
ARG SQLX_OFFLINE=true
WORKDIR /usr/src/warplink
COPY . .
RUN cargo build --release

# run
FROM rust:slim
COPY --from=builder /usr/src/warplink/target/release/warplink /usr/local/bin/warplink
CMD ["warplink"]