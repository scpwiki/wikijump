FROM redis:alpine

EXPOSE 6379
COPY ./install/files/redis/sysctl.conf /etc/sysctl.d/50-redis.conf
COPY ./install/files/redis/redis.conf /usr/local/etc/redis.conf
CMD ["redis-server", "/usr/local/etc/redis.conf"]
