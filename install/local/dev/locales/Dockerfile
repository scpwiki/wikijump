FROM python:3.9-slim

# Copy source
RUN mkdir /src
COPY ./locales /src/locales
WORKDIR /src/locales

# Install dependencies
RUN apt update && \
    apt install -y gettext
RUN pip3 install --upgrade -r requirements.txt

# Build localization files
RUN python3 -m messages .
