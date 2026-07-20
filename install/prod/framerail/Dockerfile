#
# Framerail build
#

FROM node:20-alpine

# SvelteKit materializes CSP and CSRF options while `pnpm build` runs; standing may pass `local` for CSP only.
ARG FRAMERAIL_ENV=prod
ARG FRAMERAIL_CSRF_CHECK_ORIGIN=true
ENV FRAMERAIL_ENV=${FRAMERAIL_ENV}
ENV FRAMERAIL_CSRF_CHECK_ORIGIN=${FRAMERAIL_CSRF_CHECK_ORIGIN}

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
