# Step 1: Build stage
FROM rust:alpine3.20 AS builder

# Install system dependencies (including musl, PROJ, SQLite, OpenSSL)
RUN apk update && apk add --no-cache \
    musl-dev \
    proj proj-dev \
    sqlite sqlite-dev \
    openssl-dev \
    cmake \
    pkgconfig \
    curl \
    && rm -rf /var/cache/apk/*

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock to the working directory
COPY Cargo.toml Cargo.lock ./
COPY .sqlx/ /app/.sqlx

# Create a dummy main file to cache dependencies
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# Build only the dependencies to cache them
RUN cargo build --release

# Remove the dummy main file
RUN rm src/main.rs

# Copy the source code to the working directory
COPY src ./src

# # Build the project for musl
# RUN rustup target add x86_64-unknown-linux-musl
# RUN cargo build --release --target x86_64-unknown-linux-musl

# Step 2: Final stage
FROM alpine:3.20

# Set the working directory inside the final image
WORKDIR /app

# Install required runtime dependencies (openssl, musl, etc.)
RUN apk add --no-cache \
    libgcc \
    libssl3 \
    gcompat

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/soil-api-rust /app/

# Expose port 3000
EXPOSE 3000

# Set the entrypoint command to start your Axum application
CMD ["./soil-api-rust"]
