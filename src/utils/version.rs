
pub struct Version;

static VERSION: &str = include_str!("../../Cargo.toml");

impl Version {
    pub fn get() -> &'static String {
        // From Cargo.toml:
        let v = VERSION
            .split("\n")
            .find(|line| line.starts_with("version"))
            .unwrap()
            .split("=")
            .last()
            .unwrap()
            .trim()
            .replace("\"", "");

        Box::leak(Box::new(v))
    }
}