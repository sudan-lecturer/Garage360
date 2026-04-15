FROM nginx:alpine

RUN apk add --no-cache \
    openssl

RUN mkdir -p /etc/nginx/ssl \
    /var/www/html

RUN openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
    -keyout /etc/nginx/ssl/key.pem \
    -out /etc/nginx/ssl/cert.pem \
    -subj "/CN=localhost"

COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80 443

CMD ["nginx", "-g", "daemon off;"]
