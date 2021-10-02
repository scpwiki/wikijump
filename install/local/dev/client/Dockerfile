FROM circleci/node:latest

# Copy source
RUN mkdir /tmp/src
COPY ./client ./locales ./web /tmp/src/
WORKDIR /tmp/src/client

# Install dependencies
RUN sudo npm install -g pnpm
RUN pnpm install

# Build frontend
RUN pnpm build
