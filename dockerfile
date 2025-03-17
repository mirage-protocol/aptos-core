FROM rust:1.75-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    cmake \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    git \
    clang \
    lld \
    libelf-dev \
    libdw-dev \
    elfutils \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy the entire repository
COPY . .

# List contents to debug
RUN ls -la ecosystem/indexer-grpc
RUN ls -la ecosystem/indexer-grpc/*/Cargo.toml || true
RUN find . -name "Cargo.toml" | grep indexer-grpc || true

# First attempt a workspace-level build
WORKDIR /build/ecosystem/indexer-grpc
RUN echo "Attempting to build with workspace-level cargo..." && \
    cargo build --release || echo "Workspace-level build failed, trying individual packages..."

# Attempt to build each package individually
# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-cache-worker
# RUN cargo build --release || echo "Failed to build indexer-grpc-cache-worker"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-data-service
# RUN cargo build --release || echo "Failed to build indexer-grpc-data-service"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-data-service-v2
# RUN cargo build --release || echo "Failed to build indexer-grpc-data-service-v2"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-file-store
# RUN cargo build --release || echo "Failed to build indexer-grpc-file-store"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-file-store-backfiller
# RUN cargo build --release || echo "Failed to build indexer-grpc-file-store-backfiller"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-file-checker
# RUN cargo build --release || echo "Failed to build indexer-grpc-file-checker"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-fullnode
# RUN cargo build --release || echo "Failed to build indexer-grpc-fullnode"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-manager
# RUN cargo build --release || echo "Failed to build indexer-grpc-manager"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-table-info
# RUN cargo build --release || echo "Failed to build indexer-grpc-table-info"

# WORKDIR /build/ecosystem/indexer-grpc/indexer-grpc-utils
# RUN cargo build --release || echo "Failed to build indexer-grpc-utils"

# WORKDIR /build/ecosystem/indexer-grpc/transaction-filter
# RUN cargo build --release || echo "Failed to build transaction-filter"

# Reset working directory
WORKDIR /build

# Check if any binaries were built
RUN echo "Checking for built binaries..." && \
    find /build/target/release -type f -executable | grep -v "\.d$" | sort || true
RUN find /build/ecosystem/indexer-grpc/*/target/release -type f -executable | grep -v "\.d$" | sort || true

# Create runtime image
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create directories
RUN mkdir -p /opt/aptos/file-store /opt/aptos/certs

# Copy all binaries from builder
RUN mkdir -p /usr/local/bin/
COPY --from=builder /build/target/release/* /usr/local/bin/

# List binaries to verify successful copy
RUN ls -la /usr/local/bin/

# Set working directory
WORKDIR /opt/aptos

# Default command (will be overridden by docker-compose)
CMD ["/bin/bash"]

# Expose common ports used by the services
EXPOSE 50051 50052 50053 8084 18084