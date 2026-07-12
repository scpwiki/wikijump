# Set up DNS ACME challenge providers
FROM public.ecr.aws/docker/library/caddy:builder-alpine AS builder
ARG PROVIDER=digitalocean

RUN xcaddy build --with github.com/caddy-dns/${PROVIDER}

FROM public.ecr.aws/docker/library/caddy:latest
COPY --from=builder /usr/bin/caddy /usr/bin/caddy

# Install dependencies
RUN apk add --no-cache curl jq

# Install files
COPY ./install/prod/caddy/Caddyfile /etc/caddy/Caddyfile
COPY ./install/prod/caddy/start.sh /usr/local/bin/wikijump-start-caddy
COPY ./install/prod/caddy/health-check.sh /usr/local/bin/wikijump-health-check
COPY ./install/prod/caddy/generate-caddyfile.sh /usr/local/bin/wikijump-generate-caddyfile
COPY ./install/prod/caddy/update-caddyfile.sh /usr/local/bin/wikijump-update-caddyfile
COPY ./install/prod/caddy/request.json /etc/caddy-request.json

# Set up provisional Caddyfile
# Ensure deploy host matches with request.json
RUN deploy_host="$(jq -er '.params.deploy_host | strings | select(test("^[A-Za-z0-9.-]+:[0-9]+$"))' /etc/caddy-request.json)" && \
    sed -i "s/<<DEPLOY_HOST>>/${deploy_host}/" /etc/caddy/Caddyfile && \
    ! grep -q '<<DEPLOY_HOST>>' /etc/caddy/Caddyfile && \
    caddy validate --config /etc/caddy/Caddyfile --adapter caddyfile

# Set up cron
COPY ./install/prod/caddy/crontab /etc/caddy/crontab
RUN crontab -u daemon /etc/caddy/crontab && \
    rm /etc/caddy/crontab

# Run cron and caddy
CMD ["/usr/local/bin/wikijump-start-caddy"]
