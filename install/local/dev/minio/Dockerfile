FROM quay.io/minio/mc:latest AS mc
FROM quay.io/minio/minio:latest AS minio

COPY --from=mc /usr/bin/mc /usr/local/bin/mc
COPY ./docker-entrypoint.sh ./healthcheck.sh /

ENTRYPOINT ["/docker-entrypoint.sh"]
