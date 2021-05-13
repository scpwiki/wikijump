FROM node:alpine as npm

# Please note this arg is also defined in the nginx layer further down.
ARG WIKIJUMP_REPO_DIR="wikijump"

ARG WIKIJUMP_REPO_BRANCH="develop"
ARG WIKIJUMP_REPO="https://github.com/scpwiki/wikijump.git"
ARG WIKIJUMP_DIR="/var/www/${WIKIJUMP_REPO_DIR}"
WORKDIR /src

RUN apk add --no-cache git

RUN git clone \
            --depth 10 \
            --branch "${WIKIJUMP_REPO_BRANCH}" \
            "${WIKIJUMP_REPO}"

RUN npm install pnpm --global

# Install client monorepo dependencies, and build modules
WORKDIR /src/${WIKIJUMP_REPO_DIR}/client
RUN pnpm install
RUN pnpm build:remote

# Install frontend dependencies, and build
WORKDIR /src/${WIKIJUMP_REPO_DIR}/web
RUN pnpm install
RUN pnpm build:remote

FROM nginx:alpine
ARG WIKIJUMP_REPO_DIR="wikijump"
EXPOSE 80

COPY --from=npm /src/${WIKIJUMP_REPO_DIR}/web/web /var/www/wikijump/web/web
COPY nginx.conf /etc/nginx/nginx.conf
