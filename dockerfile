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

# # List contents to debug
# RUN ls -la ecosystem/indexer-grpc
# RUN ls -la ecosystem/indexer-grpc/*/Cargo.toml || true
# RUN find . -name "Cargo.toml" | grep indexer-grpc || true

# # First attempt a workspace-level build
# RUN echo "Attempting to build with workspace-level cargo..." && \
#     cargo build --release || echo "Workspace-level build failed, trying individual packages..."

# Attempt to build each package individually
RUN cargo build -p aptos-indexer-grpc-cache-worker --release

RUN cargo build -p aptos-indexer-grpc-data-service --release

# RUN cargo build -p aptos-indexer-grpc-data-service-v2 --release || echo "Failed to build indexer-grpc-data-service-v2"

RUN cargo build -p aptos-indexer-grpc-file-store --release

# RUN cargo build -p aptos-indexer-grpc-file-store-backfiller --release || echo "Failed to build indexer-grpc-file-store-backfiller"

# RUN cargo build -p aptos-indexer-grpc-file-checker --release || echo "Failed to build indexer-grpc-file-checker"

# RUN cargo build -p aptos-indexer-grpc-fullnode --release || echo "Failed to build indexer-grpc-fullnode"

# RUN cargo build -p aptos-indexer-grpc-manager --release || echo "Failed to build indexer-grpc-manager"

# RUN cargo build -p aptos-indexer-grpc-table-info --release || echo "Failed to build indexer-grpc-table-info"

# RUN cargo build -p aptos-indexer-grpc-utils --release || echo "Failed to build indexer-grpc-utils"

# RUN cargo build -p aptos-transaction-filter --release || echo "Failed to build transaction-filter"

# # Reset working directory
# WORKDIR /build

# Check if any binaries were built
# RUN echo "Checking for built binaries..." && \
#     find /build/target/release -type f -executable | grep -v "\.d$" | sort || true
# RUN find /build/ecosystem/indexer-grpc/*/target/release -type f -executable | grep -v "\.d$" | sort || true

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
# RUN ls -la /usr/local/bin/

# Set working directory
WORKDIR /opt/aptos

# Expose common ports used by the services
EXPOSE 50051 50052 50053 8084 18084

# Default command (will be overridden by docker-compose)
CMD ["/bin/bash"]