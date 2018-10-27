# We don't use a multi-stage build, as it prevents using a Docker volume
# for the ~/.cargo cache (see Makefile)

FROM scratch

WORKDIR /risso
COPY target/x86_64-unknown-linux-musl/release/risso_actix .
CMD ["/risso/risso_actix"]
