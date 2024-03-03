use bollard::container::LogOutput;
use bollard::exec::StartExecResults;
use bollard::exec::{CreateExecOptions, StartExecOptions};
use futures_util::stream::StreamExt;
use std::default::Default;

use crate::DOCKER;

pub async fn exec(container_id: &str, cmd: &[&str]) -> (String, i64, String) {
    let config = CreateExecOptions {
        cmd: Some(cmd.to_owned()),
        // working_dir: Some("/"),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let results = match DOCKER.create_exec(container_id, config).await {
        Ok(exec) => exec,
        Err(e) => {
            return ("".to_owned(), 1.into(), e.to_string());
        }
    };

    let start_options = Some(StartExecOptions {
        detach: false,
        ..Default::default()
    });

    let output_stream = DOCKER
        .start_exec(&results.id, start_options)
        .await
        .expect("Error starting exec");

    let mut out = String::new();
    let mut error = String::new();
    if let StartExecResults::Attached { mut output, .. } = output_stream {
        while let Some(Ok(msg)) = output.next().await {
            match msg {
                LogOutput::StdOut { message } => {
                    out.push_str(String::from_utf8_lossy(&message).as_ref());
                }
                LogOutput::StdErr { message } => {
                    error.push_str(String::from_utf8_lossy(&message).as_ref());
                }
                _ => {}
            }
        }
    }

    let exec_inspect = DOCKER
        .inspect_exec(&results.id)
        .await
        .expect("Error inspecting exec");

    let Some(exit_code) = exec_inspect.exit_code else {
        return (out, 1.into(), error);
    };

    (out, exit_code, error)
}
