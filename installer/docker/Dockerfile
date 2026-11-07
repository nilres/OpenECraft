# Images of the OpenECraft parts server and project server, built from the repository root:
#
#   docker build -f installer/docker/Dockerfile --target parts-server -t openecraft-parts-server .
#   docker build -f installer/docker/Dockerfile --target project-server -t openecraft-project-server .
#
# CI builds them on every push and publishes them to ghcr.io/<owner>/openecraft-parts-server
# and ghcr.io/<owner>/openecraft-project-server on master and for release tags
# (docs/installation.md). The Rust version is the one pinned in .github/workflows/ci.yml,
# cargo-about the one of the installer scripts (docs/releasing.md).

ARG RUST_VERSION=1.97.0

FROM rust:${RUST_VERSION}-bookworm AS tools
RUN cargo install cargo-about@0.9.3 --locked --features cli

FROM rust:${RUST_VERSION}-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p openecraft-parts-server -p openecraft-project-server \
    && mkdir /out \
    && cp target/release/openecraft-parts-server target/release/openecraft-project-server /out/
# The third-party licenses of the whole workspace, as in every other package.
COPY --from=tools /usr/local/cargo/bin/cargo-about /usr/local/cargo/bin/
RUN cargo fetch --locked \
    && cargo about generate --fail -c installer/about.toml -o /out/THIRD-PARTY-LICENSES.html installer/about.hbs

# ca-certificates for sign-in with LDAP directories over TLS. Both servers run as the
# user openecraft and keep their data in the volume /data.
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --user-group --no-create-home --home-dir /data openecraft \
    && install -d -o openecraft -g openecraft /data
COPY LICENSE-MIT LICENSE-APACHE /usr/share/doc/openecraft/
COPY --from=build /out/THIRD-PARTY-LICENSES.html /usr/share/doc/openecraft/
WORKDIR /data
VOLUME /data
USER openecraft

# Data portal: parts-db.sqlite and parts-db.sqlite.access.json in /data. The settings
# come from the configuration file below (mount your own there, or give options on the
# command line: `docker run … openecraft-parts-server --install-catalogs all`); the
# administrator token is read from OPENECRAFT_TOKEN.
FROM runtime AS parts-server
COPY --from=build /out/openecraft-parts-server /usr/local/bin/
COPY installer/docker/parts-server.toml /etc/openecraft/parts-server.toml
ENV OPENECRAFT_PARTS_SERVER_CONFIG=/etc/openecraft/parts-server.toml
EXPOSE 8090
ENTRYPOINT ["openecraft-parts-server"]

# Project server: the project files, users.json and access.json in /data; the access
# token is read from OPENECRAFT_PROJECT_TOKEN.
FROM runtime AS project-server
COPY --from=build /out/openecraft-project-server /usr/local/bin/
COPY installer/docker/project-server.toml /etc/openecraft/project-server.toml
ENV OPENECRAFT_PROJECT_SERVER_CONFIG=/etc/openecraft/project-server.toml
EXPOSE 8091
ENTRYPOINT ["openecraft-project-server"]
