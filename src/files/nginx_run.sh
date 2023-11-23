#!/bin/sh

docker run -d \
--name nbot_nginx \
-p 443:443 \
-p 0.0.0.0:80:80 \
-v {{volume_dir}}/certs:/etc/letsencrypt:rw \
-v {{volume_dir}}/conf.d:/etc/nginx/conf.d \
-v {{volume_dir}}/html:/usr/share/nginx/html \
-v {{volume_dir}}/static:/static/ \
-v {{volume_dir}}/media:/media/ \
nbot/nginx