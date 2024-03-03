use bollard::volume::RemoveVolumeOptions;

use crate::DOCKER;

pub async fn find_by_container(container: &str) -> Vec<String> {
    let details = DOCKER.inspect_container(container, None).await;
    match details {
        Ok(details) => {
            let mut volumes = vec![];
            if let Some(mounts) = details.mounts {
                for mount in mounts {
                    if let Some(name) = mount.name {
                        volumes.push(name.clone());
                    }
                }
            };
            volumes
        }
        Err(_) => vec![],
    }
}

pub async fn remove_many(volumes: Vec<String>, force: bool) {
    let options = Some(RemoveVolumeOptions { force });

    for volume in volumes {
        if DOCKER.remove_volume(&volume, options).await.is_ok() {
            println!("{volume}");
        }
    }
}
