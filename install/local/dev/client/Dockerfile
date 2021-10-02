FROM circleci/node:latest

# Copy source
RUN sudo mkdir /src
COPY ./client /src/client
COPY ./locales /src/locales
COPY ./web /src/web
RUN sudo chown -R circleci /src
WORKDIR /src/client

# Install dependencies
RUN sudo npm install -g pnpm
RUN pnpm install

# Build frontend
RUN pnpm build
