FROM rust:1.85.0-slim as builder

WORKDIR /usr/src/app
COPY . .

# Install dependencies and build the application
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

# Create a smaller production image
FROM debian:bookworm-slim

# Install necessary runtime libraries
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/network-sync-server .

# Expose the websocket port
EXPOSE 8080

# Command to run the server
CMD ["./network-sync-server", "--port", "8080"]
