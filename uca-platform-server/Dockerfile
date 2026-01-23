FROM rust:1.93.0-alpine AS build

RUN apk add --no-cache \
    build-base \
    ca-certificates \
    libreoffice \
    ttf-dejavu \
    openssl-dev \
    openssl-libs-static \
    pkgconfig \
    openjdk17-jre \
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
    openssl \
    openjdk17-jre \
    fontconfig

WORKDIR /app
COPY --from=build /app/target/release/ucaplatform /app/ucaplatform
COPY --from=build /data /data
COPY --from=build /app/fonts /usr/share/fonts/Windows

ENV LIBREOFFICE_PATH=/usr/bin/soffice
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk
EXPOSE 8443
ENTRYPOINT ["/app/ucaplatform"]
