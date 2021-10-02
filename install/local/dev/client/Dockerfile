FROM circleci/node:latest

# Copy source
RUN mkdir /src
COPY ./client /src/client
COPY ./web /src/web
WORKDIR /src/client

# Install and build
RUN npm install -g pnpm
RUN pnpm install
RUN pnpm build
