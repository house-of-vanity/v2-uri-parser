use clap::{value_parser, Arg, Command};
pub mod config_models;
mod parser;
pub mod utils;
mod xray_runner;

#[tokio::main]
async fn main() {
    let matches = Command::new("v2parser")
        .version("0.3.1")
        .about("Parses V2ray URI and generates JSON config for xray")
        .arg(
            Arg::new("uri")
                .help("V2ray URI to parse")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("socksport")
                .long("socksport")
                .help("Optional SOCKS5 proxy port for inbound")
                .value_name("PORT")
                .value_parser(value_parser!(u16)),
        )
        .arg(
            Arg::new("httpport")
                .long("httpport")
                .help("Optional HTTP proxy port for inbound")
                .value_name("PORT")
                .value_parser(value_parser!(u16)),
        )
        .arg(
            Arg::new("get_metadata")
                .long("get-metadata")
                .help("Only print config meta data")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("run")
                .long("run")
                .help("Run xray-core with the generated config")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("xray_binary")
                .long("xray-binary")
                .help("Path to xray-core binary (default: xray from PATH)")
                .value_name("PATH"),
        )
        .get_matches();

    let uri = matches.get_one::<String>("uri").unwrap();
    let socksport = matches.get_one::<u16>("socksport").copied();
    let httpport = matches.get_one::<u16>("httpport").copied();
    let get_metadata = matches.get_flag("get_metadata");
    let run_mode = matches.get_flag("run");
    let xray_binary = matches.get_one::<String>("xray_binary").map(|s| s.as_str()).unwrap_or("xray-core");

    if get_metadata {
        print!("{}", parser::get_metadata(uri));
        return;
    }

    let json_config = parser::create_json_config(uri, socksport, httpport);

    if run_mode {
        // Run mode: start xray-core with the config
        let mut runner = xray_runner::XrayRunner::new();

        match runner.start(&json_config, xray_binary).await {
            Ok(()) => {
                println!("xray-core started successfully. Press Ctrl+C to stop.");

                // Wait for shutdown signal
                xray_runner::wait_for_shutdown_signal().await;

                // Stop xray-core
                if let Err(e) = runner.stop().await {
                    eprintln!("Error stopping xray-core: {}", e);
                } else {
                    println!("xray-core stopped successfully.");
                }
            }
            Err(e) => {
                eprintln!("Failed to start xray-core: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Normal mode: just print the config
        println!("{}", json_config);
    }
}
