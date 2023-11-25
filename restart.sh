cargo run -- down
sudo rm -rf ~/.config/
docker rmi nbot/nginx
cargo run -- nginx run
docker exec nbot_nginx cat /etc/nginx/conf.d/default.conf