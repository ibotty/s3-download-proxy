FROM registry.access.redhat.com/ubi9-minimal
ARG BINARY=target/release/s3-download-proxy

LABEL maintainer="Tobias Florek <tob@butter.sh>"

EXPOSE 8080/tcp 12345/tcp

COPY $BINARY /s3-download-proxy
COPY samples/default.config /etc/s3-download-proxy.config

CMD ["/s3-download-proxy", "-c", "/etc/s3-download-proxy.config"]
USER 1000
