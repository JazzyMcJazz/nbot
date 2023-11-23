#!/bin/sh

echo "Certbot entrypoint is running."

mv default.conf /etc/nginx/conf.d/default >> /dev/null 2>&1

crontab scheduler.txt
crontab -l

function inotifywait_listen() {
    while inotifywait -e create -e modify -e delete -e move /etc/nginx/conf.d; do
        echo "Configuration change detected, reloading Nginx..."
        nginx -s reload
    done
}

# Monitor the conf.d directory for changes and reload Nginx when a change is detected
inotifywait_listen &

# Start Nginx
nginx -g 'daemon off;'