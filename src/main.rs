use zecwalletlitelib::lightclient::{self, LightClientConfig};
use zecwalletlitelib::startup_helpers::{report_permission_error,
                                        startup,
                                        start_interactive,
                                        attempt_recover_seed};
use log::error;

pub fn main() {
    // Get command line arguments
    use clap::{Arg, App};
    let matches = App::new("Zecwallet CLI")
                    .version("1.0.0")
                    .arg(Arg::with_name("plantseed")
                        .short("s")
                        .long("plantseed")
                        .value_name("seed_phrase")
                        .help("Create a new wallet with the given 24-word seed phrase. Will fail if wallet already exists")
                        .takes_value(true))
                    .arg(Arg::with_name("server")
                        .long("server")
                        .value_name("server")
                        .help("Lightwalletd server to connect to.")
                        .takes_value(true)
                        .default_value(lightclient::DEFAULT_SERVER))
                    .arg(Arg::with_name("dangerous")
                        .long("dangerous")
                        .help("Disable server TLS certificate verification. Use this if you're running a local lightwalletd with a self-signed certificate. WARNING: This is dangerous, don't use it with a server that is not your own.")
                        .takes_value(false))
                    .arg(Arg::with_name("recover")
                        .long("recover")
                        .help("Attempt to recover the seed from the wallet")
                        .takes_value(false))
                    .arg(Arg::with_name("nosync")
                        .help("By default, zecwallet-cli will sync the wallet at startup. Pass --nosync to prevent the automatic sync at startup.")
                        .long("nosync")
                        .short("n")
                        .takes_value(false))
                    .arg(Arg::with_name("COMMAND")
                        .help("Command to execute. If a command is not specified, zecwallet-cli will start in interactive mode.")
                        .required(false)
                        .index(1))
                    .arg(Arg::with_name("PARAMS")
                        .help("Params to execute command with. Run the 'help' command to get usage help.")
                        .required(false)
                        .multiple(true))
                    .get_matches();

    if matches.is_present("recover") {
        attempt_recover_seed();
        return;
    }

    let command = matches.value_of("COMMAND");
    let params = matches.values_of("PARAMS").map(|v| v.collect()).or(Some(vec![])).unwrap();

    let maybe_server  = matches.value_of("server").map(|s| s.to_string());
    let seed          = matches.value_of("seed").map(|s| s.to_string());

    let server = LightClientConfig::get_server_or_default(maybe_server);

    // Test to make sure the server has all of scheme, host and port
    if server.scheme_str().is_none() || server.host().is_none() || server.port_part().is_none() {
        eprintln!("Please provide the --server parameter as [scheme]://[host]:[port].\nYou provided: {}", server);
        return;
    }

    let dangerous = matches.is_present("dangerous");
    let nosync = matches.is_present("nosync");
    let (command_tx, resp_rx) = match startup(server, dangerous, seed, !nosync, command.is_none()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error during startup: {}", e);
            error!("Error during startup: {}", e);
            match e.raw_os_error() {
                Some(13) => report_permission_error(),
                _        => eprintln!("Something else!")
            }
            return;
        }
    };

    if command.is_none() {
        start_interactive(command_tx, resp_rx);
    } else {
        command_tx.send(
            (command.unwrap().to_string(),
                params.iter().map(|s| s.to_string()).collect::<Vec<String>>()))
            .unwrap();

        match resp_rx.recv() {
            Ok(s) => println!("{}", s),
            Err(e) => {
                let e = format!("Error executing command {}: {}", command.unwrap(), e);
                eprintln!("{}", e);
                error!("{}", e);
            }
        }

        // Save before exit
        command_tx.send(("save".to_string(), vec![])).unwrap();
        resp_rx.recv().unwrap();
    }
}
