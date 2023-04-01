#
# nginx reverse proxy
#

FROM nginx:alpine

RUN apk add --no-cache curl

COPY ./install/files/nginx/nginx.conf /etc/nginx/nginx.conf
COPY ./install/files/nginx/proxy.conf /etc/nginx/proxy.conf
COPY ./install/files/nginx/gzip.conf /etc/nginx/gzip.conf
COPY ./install/files/nginx/ffdhe2048.txt /etc/nginx/ffdhe2048.txt

EXPOSE 80
EXPOSE 443
