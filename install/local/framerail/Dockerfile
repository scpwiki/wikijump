#
# Framerail build
#

FROM node:20-alpine

RUN apk add --no-cache curl
RUN npm install -g pnpm
COPY framerail /app
COPY assets /app/src/assets
WORKDIR /app

# Install files
COPY ./install/local/framerail/health-check.sh /usr/local/bin/wikijump-health-check
COPY ./install/local/framerail/framerail-start /usr/local/bin/framerail-start

# Install node dependencies
RUN pnpm install

EXPOSE 3393
ENV FRAMERAIL_ENV=local
CMD ["framerail-start"]
