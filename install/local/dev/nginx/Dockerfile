# Intermediate images
FROM scpwiki/client:local AS client

# Main image
FROM nginx:alpine
EXPOSE 80

# Build variables
ARG ENVIRONMENT="local/dev"
ARG BUILD_DIR="./install/${ENVIRONMENT}/nginx"
ARG WIKIJUMP_DIR="/var/www/wikijump"

# Copy Wikijump repository
COPY ./web ${WIKIJUMP_DIR}/web

# Copy sources from intermediate images
COPY --from=client /src/web/web /var/www/wikijump/web/web
COPY ${BUILD_DIR}/nginx.conf /etc/nginx/nginx.conf
