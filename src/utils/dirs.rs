use std::fs;

use directories::ProjectDirs;

pub struct Dirs;

impl Dirs {
    fn dir() -> String {
        let dirs = ProjectDirs::from("dev", "treeleaf", "nbot").unwrap();
        let config_dir = dirs.config_dir();
        if fs::read_dir(config_dir).is_err() {
            fs::create_dir_all(config_dir).expect("Failed to create config directory");
        }
        config_dir
            .to_str()
            .expect("Failed to convert config directory to string")
            .to_owned()
    }

    pub fn _config_dir() -> String {
        Self::dir()
    }

    pub fn config_file() -> String {
        let config_dir = Self::dir();
        format!("{}/config.json", config_dir)
    }

    pub fn temp() -> String {
        let config_dir = Self::dir();
        let temp = format!("{}/temp", config_dir);
        if fs::read_dir(&temp).is_err() {
            fs::create_dir_all(&temp).expect("Failed to create temp directory");
        }
        temp
    }

    pub fn rm_temp() {
        let temp = Self::temp();
        fs::remove_dir_all(temp).expect("Failed to remove temp directory");
    }

    pub fn _nginx() -> String {
        let config_dir = Self::dir();
        let nginx = format!("{}/nginx", config_dir);
        if fs::read_dir(&nginx).is_err() {
            fs::create_dir_all(&nginx).expect("Failed to create nginx directory");
        }
        nginx
    }

    pub fn nginx_volumes() -> String {
        let config_dir = Self::dir();
        let nginx_volumes = format!("{}/nginx/volumes", config_dir);
        if fs::read_dir(&nginx_volumes).is_err() {
            fs::create_dir_all(&nginx_volumes).expect("Failed to create nginx volumes directory");
        }
        nginx_volumes
    }

    pub fn nginx_certs() -> String {
        let nginx = Self::nginx_volumes();
        let certs = format!("{}/certs", nginx);
        if fs::read_dir(&certs).is_err() {
            fs::create_dir_all(&certs).expect("Failed to create nginx certs directory");
        }
        certs
    }

    pub fn nginx_confd() -> String {
        let nginx = Self::nginx_volumes();
        let confd = format!("{}/conf.d", nginx);
        if fs::read_dir(&confd).is_err() {
            fs::create_dir_all(&confd).expect("Failed to create nginx conf.d directory");
        }
        confd
    }

    fn _nginx_html() -> String {
        let nginx = Self::nginx_volumes();
        let html = format!("{}/html", nginx);
        if fs::read_dir(&html).is_err() {
            fs::create_dir_all(&html).expect("Failed to create nginx html directory");
        }
        html
    }

    fn nginx_static() -> String {
        let nginx = Self::nginx_volumes();
        let static_dir = format!("{}/static", nginx);
        if fs::read_dir(&static_dir).is_err() {
            fs::create_dir_all(&static_dir).expect("Failed to create nginx static directory");
        }
        static_dir
    }

    fn nginx_media() -> String {
        let nginx = Self::nginx_volumes();
        let media = format!("{}/media", nginx);
        if fs::read_dir(&media).is_err() {
            fs::create_dir_all(&media).expect("Failed to create nginx media directory");
        }
        media
    }

    pub fn init_volumes() {
        // Called in order to create the directories if they don't exist
        let _ = Self::nginx_certs();
        let _ = Self::nginx_confd();
        // let _ = Self::nginx_html();
        let _ = Self::nginx_static();
        let _ = Self::nginx_media();
    }
}
