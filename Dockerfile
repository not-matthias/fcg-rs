FROM rustlang/rust:nightly as builder

# Build the project
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

# Run the project
FROM debian:bullseye-slim
COPY --from=builder /usr/local/cargo/bin/fcg /usr/local/bin/app
ENTRYPOINT ["/usr/local/bin/app"]
