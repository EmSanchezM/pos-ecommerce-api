#stage 1
FROM rust:1.92.0 as builder
WORKDIR /app
COPY . .
RUN cargo build --release


#stage 2
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/api-gateway .
EXPOSE 8000
CMD ["./api-gateway"]