FROM redis:alpine

EXPOSE 6379
COPY ./redis.conf /usr/local/etc/redis.conf
CMD ["redis-server", "/usr/local/etc/redis.conf"]
