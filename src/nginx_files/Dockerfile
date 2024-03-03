FROM nginx:alpine

RUN apk update
RUN apk add certbot certbot-nginx inotify-tools openssl

RUN mkdir /etc/letsencrypt

COPY entrypoint.sh entrypoint.sh
COPY scheduler.txt scheduler.txt
COPY default.conf default.conf
COPY template.conf template.conf
COPY functions.sh functions.sh

RUN chmod +x entrypoint.sh
RUN chmod +x functions.sh

EXPOSE 80 443

ENTRYPOINT ["sh", "entrypoint.sh"]