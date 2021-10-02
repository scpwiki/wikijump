FROM python:3.9-slim

# Copy source
RUN mkdir /src
COPY ./locales /src/locales
WORKDIR /src/locales

# Build localization files
RUN python3 -m messages .
