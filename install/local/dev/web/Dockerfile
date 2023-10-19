#
# Framerail build
#

FROM node:19-alpine

RUN apk add --no-cache curl g++ make python3
RUN npm install -g pnpm
COPY framerail /app
WORKDIR /app

RUN pnpm install

EXPOSE 3000
CMD ["pnpm", "dev"]
