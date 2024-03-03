#!/bin/sh

function inotifywait_listen() {
    while inotifywait -e create -e modify -e delete -e move /etc/nginx/conf.d; do
        echo "Configuration change detected, reloading Nginx..."
        nginx -s reload
    done
}

echo "Nginx entrypoint"

cp default.conf /etc/nginx/conf.d/default.conf >> /dev/null 2>&1

# Create a crontab file
crontab scheduler.txt
crontab -l

# Start the cron daemon
crond

# Monitor the conf.d directory for changes and reload Nginx when a change is detected
inotifywait_listen &

# Start Nginx
nginx -g 'daemon off;'
