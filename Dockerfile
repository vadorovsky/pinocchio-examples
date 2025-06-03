FROM docker.io/ubuntu:24.04

RUN apt update && apt install -y build-essential curl \
    && sh -c "$(curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs)" -- -y \
    && sh -c "$(curl -sSfL https://release.anza.xyz/v2.2.14/install)"

ENV PATH="/root/.local/share/solana/install/active_release/bin:/root/.cargo/bin:${PATH}"
WORKDIR /src
