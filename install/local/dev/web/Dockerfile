#
# Framerail build
#

FROM node:19-alpine

# Install pnpm
RUN npm install -g pnpm

# Copy and build application
COPY framerail /app
WORKDIR /app
RUN pnpm install
RUN pnpm build

CMD ["pnpm", "dev"]
