FROM nginx:alpine

RUN apk update
RUN apk add certbot certbot-nginx

RUN mkdir /etc/letsencrypt

COPY nbotnginx_entrypoint.sh entrypoint.sh
COPY nbotnginx_scheduler.txt scheduler.txt

ENTRYPOINT ["sh", "entrypoint.sh"]