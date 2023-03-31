#
# Framerail build
#

# Install pnpm
FROM node:19-alpine AS base

RUN npm install -g pnpm
RUN mkdir /app
WORKDIR /app

# Run development server
COPY framerail/package.json framerail/pnpm-lock.yaml ./
RUN pnpm install

CMD ["pnpm", "dev"]
