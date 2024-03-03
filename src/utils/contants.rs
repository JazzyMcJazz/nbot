pub const NGINX_CERT_VOLUME: &str = "nbot_certs:/etc/letsencrypt:rw";
pub const NGINX_CONFD_VOLUME: &str = "nbot_confd:/etc/nginx/conf.d";
pub const NGINX_HTML_VOLUME: &str = "nbot_html:/usr/share/nginx/html";
pub const NGINX_STATIC_VOLUME: &str = "nbot_static:/static/";
pub const NGINX_MEDIA_VOLUME: &str = "nbot_media:/media/";
pub const NGINX_CONTAINER_NAME: &str = "nginx";
pub const NGINX_IMAGE_NAME: &str = "nbot/nginx";
