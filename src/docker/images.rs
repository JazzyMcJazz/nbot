use bollard::{
    image::{BuildImageOptions, CreateImageOptions, ListImagesOptions},
    secret::ImageSummary,
};
use futures_util::stream::StreamExt;
use std::collections::HashMap;

use crate::{
    nginx_files::{self as f},
    utils::tarball::Tarball,
    DOCKER,
};

pub async fn find_by_name(image_name: &str, tag: Option<&str>) -> Option<ImageSummary> {
    let name = if let Some(tag) = &tag {
        format!("{}:{}", image_name, tag)
    } else {
        image_name.to_owned()
    };

    let mut filters = HashMap::new();
    filters.insert("reference", vec![name.as_str()]);

    let options = Some(ListImagesOptions {
        all: true,
        filters,
        ..Default::default()
    });

    let images = DOCKER.list_images(options).await.unwrap();

    match &images.len() {
        1 => Some(images[0].clone()),
        _ => {
            let Some(mut found) = images.first() else {
                return None;
            };

            for image in &images {
                if image.repo_tags.contains(&format!("{}:latest", &image_name)) {
                    found = &image;
                    break;
                }

                if image.created > found.created {
                    found = &image;
                }
            }
            Some(found.clone())
        }
    }
}

pub async fn remove(image_id: &str) -> bool {
    let result = DOCKER.remove_image(image_id, None, None).await;
    match result {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error removing image: {}", e);
            false
        }
    }
}

pub async fn build_nginx() {
    let files: Vec<(&str, &str)> = vec![
        ("Dockerfile", f::NGINX_DOCKERFILE),
        ("entrypoint.sh", f::NGINX_ENTRYPOINT),
        ("scheduler.txt", f::NGINX_SCHEDULER),
        ("default.conf", f::NGINX_DEFAULT_CONF),
        ("template.conf", f::NGINX_TEMPLATE_CONF),
        ("functions.sh", f::NGINX_FUNCTIONS),
    ];

    let tarball = Tarball::create(files).expect("Error creating tarball");

    let options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: "nbot/nginx:latest",
        rm: true,
        ..Default::default()
    };

    let mut stream = DOCKER.build_image(options, None, Some(tarball.into()));

    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(output) => {
                if let Some(output) = output.stream {
                    print!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Error building image: {}", e);
                std::process::exit(1);
            }
        }
    }
}

pub async fn try_find_or_pull(image_name: &str, tag: Option<&str>) -> Option<ImageSummary> {
    let image = find_by_name(image_name, tag).await;
    if let Some(image) = image {
        return Some(image);
    }

    let (image_name, tag) = match image_name.split_once(':') {
        Some((name, tag)) => (name, tag),
        None => (image_name, "latest"),
    };
    let options = Some(CreateImageOptions {
        from_image: image_name,
        tag,

        ..Default::default()
    });

    println!("Pulling image: {}", image_name);
    let mut stream = DOCKER.create_image(options, None, None);
    while let Some(result) = stream.next().await {
        match result {
            Ok(_) => {}
            Err(_) => return None,
        }
    }

    find_by_name(image_name, Some(tag)).await
}
