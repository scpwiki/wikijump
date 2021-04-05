FROM nginx:alpine
EXPOSE 80
RUN apk add shadow && usermod -u 1000 nginx && groupmod -g 1000 nginx
COPY nginx.conf /etc/nginx/nginx.conf
