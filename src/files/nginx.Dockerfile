FROM nginx:alpine

RUN apk update
RUN apk add certbot certbot-nginx inotify-tools openssl

RUN mkdir /etc/letsencrypt

COPY entrypoint.sh entrypoint.sh
COPY scheduler.txt scheduler.txt
COPY default.conf default.conf

ENTRYPOINT ["sh", "entrypoint.sh"]