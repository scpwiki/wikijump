#
# Framerail build
#

FROM node:22-alpine

RUN apk add --no-cache curl
RUN npm install -g pnpm
COPY framerail /app
COPY assets /app/src/assets
WORKDIR /app

# Install files
COPY ./install/local/framerail/health-check.sh /usr/local/bin/wikijump-health-check

# Install node dependencies
RUN pnpm install

EXPOSE 3393
ENV FRAMERAIL_ENV=local
CMD ["pnpm", "dev"]
