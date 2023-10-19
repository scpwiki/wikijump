#
# Framerail build
#

FROM node:20-alpine

RUN apk add --no-cache curl
RUN npm install -g pnpm
COPY framerail /app
WORKDIR /app

RUN pnpm install

EXPOSE 3000
CMD ["pnpm", "dev"]
