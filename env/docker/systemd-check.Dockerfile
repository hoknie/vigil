FROM debian:13

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        systemd systemd-sysv procps iproute2 netcat-openbsd libcap2-bin nftables iptables \
    && rm -rf /var/lib/apt/lists/*

CMD ["/sbin/init"]
