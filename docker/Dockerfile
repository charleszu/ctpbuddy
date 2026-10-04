FROM rust:1.98-bookworm AS core-builder
WORKDIR /src
COPY core/Cargo.toml core/Cargo.lock ./core/
COPY core/ctpbuddy-wire ./core/ctpbuddy-wire
COPY core/ctpbuddy-market ./core/ctpbuddy-market
COPY core/ctpbuddy-matching ./core/ctpbuddy-matching
COPY core/ctpbuddy-ledger ./core/ctpbuddy-ledger
COPY core/ctpbuddy-server ./core/ctpbuddy-server
RUN cargo build --locked --release --manifest-path core/Cargo.toml --bin ctpbuddy-server

FROM python:3.13-slim-bookworm
WORKDIR /app
COPY --from=core-builder /src/core/target/release/ctpbuddy-server /usr/local/bin/ctpbuddy-server
COPY py /app/py
COPY refdata /app/refdata
COPY scenarios /app/scenarios
COPY docs-site-oink /app/docs-site-oink
COPY docker/acceptance.sh /usr/local/bin/ctpbuddy-docker-acceptance
RUN python -m pip wheel --no-deps --wheel-dir /tmp/wheels /app/py \
    && python -m pip install --no-cache-dir /tmp/wheels/*.whl \
    && chmod +x /usr/local/bin/ctpbuddy-docker-acceptance \
    && rm -rf /tmp/wheels /app/py
# Hugo is deliberately optional: the OINK check skips with an explicit reason if it is absent.
CMD ["ctpbuddy-docker-acceptance"]
