#
# Frontend
#

FROM circleci/node:latest AS node

# Copy source
RUN sudo mkdir /src
COPY ./ftml /src/ftml
COPY ./locales /src/locales
COPY ./web /src/web
RUN sudo chown -R circleci /src

# Install pnpm
RUN sudo npm install -g pnpm

# Build legacy frontend
WORKDIR /src/web
RUN pnpm install
RUN pnpm build:legacy

#
# Final image
#

FROM nginx:alpine
EXPOSE 80

# Build variables
ARG ENVIRONMENT="aws/dev"
ARG BUILD_DIR="./install/${ENVIRONMENT}/nginx"

# Copy Wikijump repository
COPY ./web /var/www/wikijump/web

# Copy sources from intermediate images
COPY --from=node /src/web/web /var/www/wikijump/web/web
COPY ${BUILD_DIR}/nginx.conf /etc/nginx/nginx.conf
