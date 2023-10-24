# build
FROM rust:1.71 as builder
ARG SQLX_OFFLINE=true
WORKDIR /usr/src/warplink
COPY . .
RUN cargo build --release

# run
FROM debian:bullseye-slim
COPY --from=builder /usr/src/warplink/static /etc/warplink/static
COPY --from=builder /usr/src/warplink/pages /etc/warplink/pages
COPY --from=builder /usr/src/warplink/target/release/warplink /etc/warplink/warplink
WORKDIR /etc/warplink
CMD ["/etc/warplink/warplink"]