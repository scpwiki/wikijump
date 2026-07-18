#
# Framerail build
#

FROM node:20-alpine

# SvelteKit materializes CSP while `pnpm build` runs; production callers use the default and the standing local runtime passes `local` for the documented Wikidot legacy image origin.
ARG FRAMERAIL_ENV=prod
ENV FRAMERAIL_ENV=${FRAMERAIL_ENV}

# Install dependencies
RUN apk add --no-cache curl && \
    npm install -g pnpm

# Copy sources
RUN mkdir /app
WORKDIR /app
COPY framerail/ ./
COPY assets ./src/assets

# Install app
RUN \
    pnpm install && \
    pnpm build && \
    pnpm prune --prod && \
    rm -rf src/ tests/

# Run command
USER node:node
ENV NODE_ENV=production
ENV PORT=3393
EXPOSE 3393

CMD ["/usr/local/bin/node", "build"]
