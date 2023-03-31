#
# Framerail build
#

FROM node:19-alpine

RUN npm install -g pnpm
COPY framerail /app
WORKDIR /app

RUN pnpm install

CMD ["pnpm", "dev"]
