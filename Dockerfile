# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY Cargo.toml Cargo.lock ./

# Copy the source code to the working directory
COPY src ./src

# Build the Rust project
RUN cargo build --release

# Expose the port the proxy server listens on
EXPOSE 8080

# Set environment variable for proxy credentials
ENV PROXY_CREDENTIALS="username:password"

# Run the program
CMD ["sh", "-c", "RUST_LOG=info /usr/src/app/target/release/proxy_server | tee proxy_logs.txt"]
