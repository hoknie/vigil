FROM rust:1.98.1-alpine3.22

RUN apk add --no-cache bash musl-dev just git ncurses netcat-openbsd file expect nftables audit

COPY rust-toolchain.toml /work/rust-toolchain.toml
RUN cd /work && rustup show active-toolchain

RUN rustup target add x86_64-unknown-linux-musl

RUN printf 'export PATH=/build/target/debug:/usr/local/cargo/bin:$PATH\n' > /etc/profile.d/00-rust.sh

RUN mkdir -p /var/lib/vigil/firewall /run/vigil /var/log/vigil \
    && chmod 0700 /var/lib/vigil /var/lib/vigil/firewall /run/vigil

ENV CARGO_TARGET_DIR=/build/target

ENV PATH=/build/target/debug:/usr/local/cargo/bin:$PATH

COPY env/docker/motd.sh /etc/profile.d/99-motd.sh

WORKDIR /work

CMD ["./env/docker/smoke.sh"]
