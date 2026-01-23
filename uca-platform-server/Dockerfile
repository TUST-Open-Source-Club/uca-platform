FROM rust:1.93.0-alpine AS build

RUN apk add --no-cache \
    build-base \
    ca-certificates \
    libreoffice \
    ttf-dejavu \
    fontconfig

WORKDIR /app
COPY . .

RUN cargo build --release
RUN mkdir -p /data/tls /data/uploads

FROM alpine:3.20

RUN apk add --no-cache \
    ca-certificates \
    libreoffice \
    ttf-dejavu \
    fontconfig

WORKDIR /app
COPY --from=build /app/target/release/ucaplatform /app/ucaplatform
COPY --from=build /data /data

ENV LIBREOFFICE_PATH=/usr/bin/soffice
EXPOSE 8443
ENTRYPOINT ["/app/ucaplatform"]
