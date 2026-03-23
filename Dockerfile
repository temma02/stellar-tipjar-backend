# --- Builder Stage ---
FROM rust:latest AS builder

WORKDIR /app
COPY . .

# Build for release
RUN cargo build --release

# --- Runtime Stage ---
FROM debian:bookworm-slim

# Install necessary runtime libraries
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/stellar-tipjar-backend .
COPY --from=builder /app/migrations ./migrations

# Expose port
EXPOSE 8000

# Run the app
CMD ["./stellar-tipjar-backend"]