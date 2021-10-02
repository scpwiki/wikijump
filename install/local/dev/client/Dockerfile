FROM circleci/node:latest

# Copy source
RUN sudo mkdir /src
COPY ./client ./locales ./web /src/
WORKDIR /src/client

# Install dependencies
RUN sudo npm install -g pnpm
RUN sudo pnpm install

# Build frontend
RUN pnpm build
