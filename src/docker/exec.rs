use bollard::exec::{CreateExecOptions, StartExecOptions};

use crate::DOCKER;

pub async fn exec(container_id: &str, cmd: &str) -> i64 {
    let config = CreateExecOptions {
        cmd: Some(cmd.split(" ").map(|s| s.to_string()).collect()),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    dbg!(container_id, cmd, &config);
    let result = match DOCKER.create_exec(container_id, config).await {
        Ok(exec) => exec,
        Err(e) => {
            eprintln!("Error creating exec: {}", e);
            return 1.into();
        }
    };

    let start_options = Some(StartExecOptions {
        detach: false,
        ..Default::default()
    });

    let _ = DOCKER.start_exec(&result.id, start_options)
        .await
        .expect("Error starting exec");

    let exec_inspect = DOCKER.inspect_exec(&result.id)
        .await
        .expect("Error inspecting exec");

    dbg!(&exec_inspect);

    let Some(exit_code) = exec_inspect.exit_code else {
        return 1.into();
    };

    exit_code
}