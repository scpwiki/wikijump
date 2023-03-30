#
# Framerail build
#

# Install pnpm
FROM node:19-alpine AS base

RUN npm install -g pnpm
RUN mkdir /app
WORKDIR /app

# Install dependencies
FROM base AS dependencies

COPY framerail/package.json framerail/pnpm-lock.yaml ./
RUN pnpm install

# Build application
FROM base AS build

COPY --from=dependencies /app/node_modules ./node_modules
RUN pnpm build
RUN pnpm prune --prod

# Deploy application
FROM base AS deploy

COPY --from=build /app/package.json ./dist
COPY --from=build /app/pnpm-lock.json ./dist
COPY --from=build /app/dist ./dist
COPY --from=build /app/node_modules ./node_modules

CMD ["/usr/bin/node", "dist/main.js"]
