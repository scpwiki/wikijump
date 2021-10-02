FROM circleci/node:latest

# Copy source
RUN sudo mkdir /src
COPY ./client /src/client
COPY ./ftml /src/ftml
COPY ./locales /src/locales
COPY ./web /src/web
RUN sudo chown -R circleci /src

# Install pnpm
RUN sudo npm install -g pnpm

# Build legacy frontend
WORKDIR /src/web
RUN pnpm install
RUN pnpm build:legacy

# Build main frontend
WORKDIR /src/client
RUN pnpm install
RUN pnpm build
