FROM rust:1.93.0-trixie AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libreoffice \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

RUN cargo build --release
RUN mkdir -p /data/tls /data/uploads

FROM gcr.io/distroless/base-debian12

WORKDIR /app
COPY --from=build /app/target/release/ucaplatform /app/ucaplatform
COPY --from=build /usr/bin/soffice /usr/bin/soffice
COPY --from=build /usr/lib/libreoffice /usr/lib/libreoffice
COPY --from=build /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
COPY --from=build /lib/x86_64-linux-gnu /lib/x86_64-linux-gnu
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=build /data /data

ENV LIBREOFFICE_PATH=/usr/bin/soffice
EXPOSE 8443
USER nonroot:nonroot
ENTRYPOINT ["/app/ucaplatform"]
