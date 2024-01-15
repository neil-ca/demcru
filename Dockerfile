FROM rust:latest as chef

ENV DATABASE_URL="sqlite:data.db"

WORKDIR /app
# RUN apt update && apt install lld clang -y

COPY . .

RUN cargo build --release

# ENV SQLX_OFFLINE true

FROM debian:stable-slim AS runtime
# RUN apt update && apt install glibc-source -y
WORKDIR /app
RUN apt update
# RUN apt update && apt-get install sqlite3 -y
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates -it is needed to verify TLS certificates
# when establishing HTTPS connecion
# RUN apt-get update -y \
#     && apt-get install -y --no-install-recommends openssl ca-certificates \
#     && apt-get autoremove -y \
#     && apt-get clean -y \
#     && rm -rf /var/lib/apt/lists/*
COPY --from=chef /app/target/release/demcru demcru
COPY --from=chef /app/configuration.yaml configuration.yaml
COPY --from=chef /app/config/blog.yml config/blog.yml
COPY --from=chef /app/templates templates/
COPY --from=chef /app/static static/
COPY --from=chef /app/.env .env
COPY --from=chef /app/data.db data.db
# COPY configuration configuration
# ENV APP_ENVIRONMENT production
# ENV DATABASE_URL="postgres://postgres:password@172.17.0.3:5432/newsletter"
ENV DATABASE_URL="sqlite:data.db"
ENTRYPOINT [ "./demcru" ]
