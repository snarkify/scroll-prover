# The build step is skipped, one should build on the host first
# Use the NVIDIA CUDA runtime image for the runtime stage
FROM nvidia/cuda:12.6.0-base-ubuntu22.04

WORKDIR /snarkify-data

# Copy the build artifact from the build stage
COPY target/release/snarkify /usr/local/bin/snarkify
# Set environment variables
ENV SCROLL_PROVER_ASSETS_DIR=/snarkify-data/assets \
    RUST_MIN_STACK=100000000
# Set the entrypoint for the container
ENTRYPOINT ["snarkify"]