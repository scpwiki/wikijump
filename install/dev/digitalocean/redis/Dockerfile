FROM redis:alpine

ENV PORT=6379
EXPOSE 6379
COPY ./install/dev/digitalocean/redis/redis.conf /usr/local/etc/redis.conf
CMD ["redis-server", "/usr/local/etc/redis.conf"]
