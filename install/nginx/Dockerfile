FROM nginx:1.19.4-alpine

EXPOSE 80

# Build variables
ARG MAIN_DOMAIN="wikijump.test"
ARG FILES_DOMAIN="wjfiles.test"

# Preparation
RUN mkdir /src
WORKDIR /src

# Copy sources
COPY etc ./etc

# Generate OpenSSL key
# See https://stackoverflow.com/a/41366949
RUN openssl req \
	-x509 \
	-newkey rsa:4096 \
	-sha256 \
	-days 3650 \
	-nodes \
	-keyout cert.key \
	-out cert.crt \
	-subj "/CN=${MAIN_DOMAIN}" \
	-addext "subjectAltName=DNS:${FILES_DOMAIN},DNS:*.${MAIN_DOMAIN},DNS:*.${FILES_DOMAIN}"

# Deploy generated key
RUN install -D -m644 cert.pem /usr/local/nginx/conf/cert.pem
RUN install -D -m644 cert.key /usr/local/nginx/conf/cert.key

# Install configuration files
RUN cp -av ./etc/nginx /etc

# TODO

# Clean up
WORKDIR /var/www
RUN rm -rf /src

# Main process
CMD ["nginx", "-g", "daemon off;"]
