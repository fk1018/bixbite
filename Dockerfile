FROM debian:bookworm-slim

ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bash \
        build-essential \
        ca-certificates \
        curl \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y --profile minimal --component rustfmt

WORKDIR /workspace

CMD ["bash"]
