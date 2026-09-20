FROM pgsty/mc:latest AS mc
FROM pgsty/silo:latest AS minio

COPY --from=mc /usr/bin/mc /usr/local/bin/mc
COPY ./docker-entrypoint.sh ./healthcheck.sh /

ENTRYPOINT ["/docker-entrypoint.sh"]
