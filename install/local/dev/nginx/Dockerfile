#
# Frontend
#

FROM node:alpine AS node

# Install pnpm
RUN npm install -g pnpm

# Copy source
RUN mkdir /src
COPY ./ftml /src/ftml
COPY ./locales /src/locales
COPY ./web /src/web

# Build frontend
WORKDIR /src/web
RUN pnpm install --prod
RUN pnpm build-container

#
# Final image
#

FROM nginx:alpine
EXPOSE 80

# Build variables
ARG ENVIRONMENT="local"

# Copy Wikijump repository
COPY ./web /var/www/wikijump/web

# Copy sources from intermediate images
COPY --from=node /src/web/web /var/www/wikijump/web/web
COPY --from=node /src/web/public /var/www/wikijump/web/public
COPY ./install/files/nginx.conf /etc/nginx/nginx.conf
