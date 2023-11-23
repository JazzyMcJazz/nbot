FROM nginx:alpine

RUN apk update
RUN apk add certbot certbot-nginx inotify-tools openssl

RUN mkdir /etc/letsencrypt

COPY nbotnginx_entrypoint.sh entrypoint.sh
COPY nbotnginx_scheduler.txt scheduler.txt
COPY nbotnginx_default.conf default.conf

ENTRYPOINT ["sh", "entrypoint.sh"]