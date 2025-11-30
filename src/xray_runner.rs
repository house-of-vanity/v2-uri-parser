use std::io::Write;
use std::process::Stdio;
use tempfile::NamedTempFile;
use tokio::process::{Child, Command};

pub struct XrayRunner {
    process: Option<Child>,
    config_file: Option<NamedTempFile>,
}

impl XrayRunner {
    pub fn new() -> Self {
        Self {
            process: None,
            config_file: None,
        }
    }

    pub async fn start(
        &mut self,
        config_json: &str,
        xray_binary: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create temporary config file with .json extension
        let mut temp_file = NamedTempFile::with_suffix(".json")?;
        temp_file.write_all(config_json.as_bytes())?;
        temp_file.flush()?;

        let config_path = temp_file.path().to_path_buf();

        // Start xray-core process
        let mut cmd = Command::new(xray_binary);
        cmd.arg("-config")
            .arg(&config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let child = cmd.spawn()?;

        self.process = Some(child);
        self.config_file = Some(temp_file);

        println!("Started xray-core with config: {:?}", config_path);
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut process) = self.process.take() {
            println!("Stopping xray-core process...");

            // Try graceful shutdown first
            if (process.kill().await).is_err() {
                eprintln!("Failed to kill xray-core process gracefully");
            }

            // Wait for process to exit
            match process.wait().await {
                Ok(status) => println!("xray-core exited with status: {}", status),
                Err(e) => eprintln!("Error waiting for xray-core to exit: {}", e),
            }
        }

        // Cleanup config file
        if self.config_file.take().is_some() {
            println!("Cleaned up temporary config file");
        }

        Ok(())
    }
}

impl Drop for XrayRunner {
    fn drop(&mut self) {
        #[allow(unused_mut)]
        if let Some(mut process) = self.process.take() {
            #[cfg(unix)]
            {
                if let Some(pid) = process.id() {
                    let _ = std::process::Command::new("kill")
                        .arg("-TERM")
                        .arg(pid.to_string())
                        .output();
                }
            }
            #[cfg(windows)]
            {
                let _ = process.start_kill();
            }
        }
    }
}

pub async fn wait_for_shutdown_signal() {
    #[cfg(unix)]
    {
        use futures::stream::StreamExt;
        use signal_hook::consts::signal::*;
        use signal_hook_tokio::Signals;

        let mut signals = Signals::new([SIGINT, SIGTERM]).expect("Failed to create signals");

        while let Some(signal) = signals.next().await {
            match signal {
                SIGINT | SIGTERM => {
                    println!("\nReceived shutdown signal, stopping...");
                    break;
                }
                _ => {}
            }
        }
    }

    #[cfg(windows)]
    {
        use tokio::signal::ctrl_c;

        match ctrl_c().await {
            Ok(()) => {
                println!("\nReceived Ctrl+C, stopping...");
            }
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
            }
        }
    }
}
