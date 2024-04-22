#
# Framerail build
#

FROM node:19-alpine

# Install pnpm
RUN npm install -g pnpm

# Copy source
RUN mkdir /app
WORKDIR /app
COPY framerail/ ./

# Install app
RUN \
    pnpm install --prod && \
    pnpm build && \
    pnpm prune --prod && \
    rm -rf src/ tests/

COPY framerail/package.json framerail/pnpm-lock.yaml ./
COPY assets ./src/assets
RUN pnpm install --prod
RUN pnpm build
RUN pnpm prune --prod

# Run command
USER node:node
ENV NODE_ENV=production
EXPOSE 3000

CMD ["/usr/bin/node", "build"]
