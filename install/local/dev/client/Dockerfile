FROM circleci/node:latest

# Copy source
RUN sudo mkdir /src
COPY ./client /src/client
COPY ./web /src/web
WORKDIR /src/client

# Install dependencies
RUN sudo npm install -g pnpm
RUN sudo pnpm install

# Build frontend
RUN pnpm build
