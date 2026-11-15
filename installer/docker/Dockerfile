# Images of the OpenECraft parts server, project server, web editor and website, built
# from the repository root:
#
#   docker build -f installer/docker/Dockerfile --target parts-server -t openecraft-parts-server .
#   docker build -f installer/docker/Dockerfile --target project-server -t openecraft-project-server .
#   docker build -f installer/docker/Dockerfile --target editor-web -t openecraft-web .
#   docker build -f installer/docker/Dockerfile --target website -t openecraft-website .
#
# CI builds them on every push and publishes them to ghcr.io/<owner>/openecraft-parts-server,
# ghcr.io/<owner>/openecraft-project-server, ghcr.io/<owner>/openecraft-web and
# ghcr.io/<owner>/openecraft-website on master and for release tags (docs/installation.md). The Rust version is the one pinned in
# .github/workflows/ci.yml, cargo-about and trunk the ones of the installer scripts and
# the job `web` (docs/releasing.md).

ARG RUST_VERSION=1.97.0
ARG NGINX_VERSION=1.28

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

# The web editor: the release build of trunk (as in the job `web`), with the license
# files next to the page (installer/web/licenses.sh, run by trunk).
FROM tools AS web-build
RUN rustup target add wasm32-unknown-unknown \
    && cargo install trunk@0.22.1 --locked
WORKDIR /src
COPY . .
RUN cargo fetch --locked \
    && cd openecraft \
    && trunk build --release

# Static files served by nginx as an unprivileged user on port 8080. The page reaches the
# parts and project servers from the browser: give their addresses in the editor (the
# servers allow requests from every origin).
FROM nginxinc/nginx-unprivileged:${NGINX_VERSION}-alpine AS editor-web
COPY installer/docker/editor-web.conf /etc/nginx/conf.d/default.conf
COPY --from=web-build /src/openecraft/dist/ /usr/share/nginx/html/
EXPOSE 8080

# The website (website/, docs/website.md): the page with its live demos, and the web
# editor built once more with relative addresses for the folder /editor/ beside it.
# The WebAssembly, the scripts and the other big texts get a compressed copy beside
# them, which nginx sends as it is (gzip_static of website.conf) instead of
# compressing 40 MB for every visitor; config.js is written at every start.
FROM web-build AS website-build
RUN cd website \
    && trunk build --release \
    && cd ../openecraft \
    && trunk build --release --public-url ./ --dist /src/target/website-editor \
    && find /src/website/dist /src/target/website-editor -type f \
        \( -name '*.wasm' -o -name '*.js' -o -name '*.css' -o -name '*.json' -o -name '*.svg' -o -name 'THIRD-PARTY-LICENSES.html' \) \
        ! -name config.js -exec gzip -9 -k {} +

# Static files served by nginx as an unprivileged user on port 8080: the website at /,
# the web editor at /editor/. Where the links of the page lead and the legal notice
# of the operator are set with the variables below (website/config.js; an empty link
# takes the address named there, an empty OPENECRAFT_LEGAL_NAME hides the notice), the
# public address of the site for search engines and link previews with
# OPENECRAFT_SITE_URL: the image writes the settings of the page, the parts that need
# the address and its nginx site at every start.
FROM nginxinc/nginx-unprivileged:${NGINX_VERSION}-alpine AS website
ENV OPENECRAFT_SITE_URL= \
    OPENECRAFT_EDITOR_URL=editor/ \
    OPENECRAFT_REPOSITORY_URL=https://github.com/nilres/OpenECraft \
    OPENECRAFT_DOWNLOADS_URL= \
    OPENECRAFT_DOCUMENTATION_URL= \
    OPENECRAFT_ISSUES_URL= \
    OPENECRAFT_LEGAL_NAME= \
    OPENECRAFT_LEGAL_ADDRESS= \
    OPENECRAFT_LEGAL_EMAIL= \
    OPENECRAFT_LEGAL_PHONE= \
    OPENECRAFT_LEGAL_REPRESENTATIVE= \
    OPENECRAFT_LEGAL_REGISTER= \
    OPENECRAFT_LEGAL_VAT_ID= \
    OPENECRAFT_LEGAL_RESPONSIBLE= \
    OPENECRAFT_LEGAL_TEXT= \
    OPENECRAFT_LEGAL_PRIVACY=
COPY installer/docker/website.conf /etc/nginx/templates/default.conf.template
COPY --chmod=0755 installer/docker/website-config.sh /docker-entrypoint.d/40-openecraft-website-config.sh
COPY --chmod=0755 installer/docker/website-site.sh /docker-entrypoint.d/41-openecraft-website-site.sh
COPY --from=website-build /src/website/dist/ /usr/share/nginx/html/
COPY --from=website-build /src/target/website-editor/ /usr/share/nginx/html/editor/
EXPOSE 8080
