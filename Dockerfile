# Build Stage
FROM rust:1.80-alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml ./
# Create dummy main to pre-compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source code
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Runtime Stage (Ultra-minimal Alpine container, ~15MB)
FROM alpine:3.20

RUN apk add --no-cache ca-certificates

WORKDIR /app
COPY --from=builder /app/target/release/cf-proxy /app/cf-proxy

ENV PORT=3000
EXPOSE 3000

CMD ["/app/cf-proxy"]
