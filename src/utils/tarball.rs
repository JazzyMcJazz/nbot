use std::io::Cursor;
use tar::Builder;

pub struct Tarball;

impl Tarball {
    pub fn create(files: Vec<(&str, &str)>) -> Result<Vec<u8>, std::io::Error> {
        let buffer = Vec::new();
        let cursor = Cursor::new(buffer);
        let mut tar_builder= Builder::new(cursor);

        for (file, content) in files {
            let mut header = tar::Header::new_gnu();
            header.set_path(file).unwrap();
            header.set_size(content.as_bytes().len() as u64);
            header.set_cksum();
            tar_builder.append(&header, content.as_bytes())?;
        }
    
        tar_builder.finish().expect("Error finishing tarball");
        let tarball = tar_builder.into_inner()?.into_inner();
        Ok(tarball)
    }
}