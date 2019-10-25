use std::io::{self, Error, ErrorKind};                  // in lib.rs
use std::sync::Arc;                                     // in lib.rs
use std::sync::mpsc::{channel, Sender, Receiver};       // in lib.rs

use zecwalletlitelib::{commands, startup_helpers,       // in lib.rs, startup_helpers factored out
    lightclient::{self, LightClient, LightClientConfig},// in lib.rs added self
};

use log::{info, error, LevelFilter};                    // in lib.rs
use log4rs::append::rolling_file::RollingFileAppender;  // in lib.rs
use log4rs::encode::pattern::PatternEncoder;            // in lib.rs
use log4rs::config::{Appender, Config, Root};           // in lib.rs
use log4rs::filter::threshold::ThresholdFilter;         // in lib.rs
use log4rs::append::rolling_file::policy::compound::{   // in lib.rs
    CompoundPolicy,                                     // in lib.rs
    trigger::size::SizeTrigger,                         // in lib.rs
    roll::fixed_window::FixedWindowRoller,              // in lib.rs
};



/// Build the Logging config                             // in lib.rs                
fn get_log_config(config: &LightClientConfig) -> io::Result<Config> { // in lib.rs
    let window_size = 3; // log0, log1, log2    // in lib.rs
    let fixed_window_roller =   // in lib.rs
        FixedWindowRoller::builder().build("zecwallet-light-wallet-log{}",window_size).unwrap();     // in lib.rs
    let size_limit = 5 * 1024 * 1024; // 5MB as max log file size to roll                            // in lib.rs
    let size_trigger = SizeTrigger::new(size_limit);                                                 // in lib.rs
    let compound_policy = CompoundPolicy::new(Box::new(size_trigger),Box::new(fixed_window_roller)); // in lib.rs
// in lib.rs
    Config::builder()       // in lib.rs
        .appender(          // in lib.rs
            Appender::builder()   // in lib.rs
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Info)))  // in lib.rs
                .build(   // in lib.rs
                    "logfile",    // in lib.rs
                    Box::new(   // in lib.rs
                        RollingFileAppender::builder()   // in lib.rs
                            .encoder(Box::new(PatternEncoder::new("{d} {l}::{m}{n}")))  // in lib.rs
                            .build(config.get_log_path(), Box::new(compound_policy))?,   // in lib.rs
                    ),   // in lib.rs
                ),  p// in lib.rs
        )   // in lib.rs
        .build(   // in lib.rs
            Root::builder()    // in lib.rs
                .appender("logfile")            // in lib.rs
                .build(LevelFilter::Debug),     // in lib.rs
        )  // in lib.rs
        .map_err(|e|Error::new(ErrorKind::Other, format!("{}", e))) // in lib.rs
}
            // in lib.rs
pub fn main() {         // in lib.rs
    // Get command line arguments           // in lib.rs
    use clap::{Arg, App};// in lib.rs
    let matches = App::new("Zecwallet CLI") // in main.rs slightly modified
                    .version("1.1.0")// in lib.rs
                    .arg(Arg::with_name("seed")// in lib.rs
                        .short("s")// in lib.rs
                        .long("seed")// in lib.rs
                        .value_name("seed_phrase")// in lib.rs
                        .help("Create a new wallet with the given 24-word seed phrase. Will fail if wallet already exists")// in lib.rs
                        .takes_value(true))// in lib.rs
                    .arg(Arg::with_name("birthday")// in lib.rs
                        .long("birthday")// in lib.rs
                        .value_name("birthday")// in lib.rs
                        .help("Specify wallet birthday when restoring from seed. This is the earlist block height where the wallet has a transaction.")// in lib.rs
                        .takes_value(true))// in lib.rs
                    .arg(Arg::with_name("server")// in lib.rs
                        .long("server")// in lib.rs
                        .value_name("server")// in lib.rs
                        .help("Lightwalletd server to connect to.")// in lib.rs
                        .takes_value(true)// in lib.rs
                        .default_value(lightclient::DEFAULT_SERVER))// in lib.rs
                    .arg(Arg::with_name("dangerous")// in lib.rs
                        .long("dangerous")// in lib.rs
                        .help("Disable server TLS certificate verification. Use this if you're running a local lightwalletd with a self-signed certificate. WARNING: This is dangerous, don't use it with a server that is not your own.")// in lib.rs
                        .takes_value(false))// in lib.rs
                    .arg(Arg::with_name("recover")// in lib.rs
                        .long("recover")// in lib.rs
                        .help("Attempt to recover the seed from the wallet")// in lib.rs
                        .takes_value(false))// in lib.rs
                    .arg(Arg::with_name("nosync")// in lib.rs
                        .help("By default, zecwallet-cli will sync the wallet at startup. Pass --nosync to prevent the automatic sync at startup.")// in lib.rs
                        .long("nosync")// in lib.rs
                        .short("n")// in lib.rs
                        .takes_value(false))// in lib.rs
                    .arg(Arg::with_name("COMMAND")// in lib.rs
                        .help("Command to execute. If a command is not specified, zecwallet-cli will start in interactive mode.")// in lib.rs
                        .required(false)// in lib.rs
                        .index(1))// in lib.rs
                    .arg(Arg::with_name("PARAMS")// in lib.rs
                        .help("Params to execute command with. Run the 'help' command to get usage help.")// in lib.rs
                        .required(false)// in lib.rs
                        .multiple(true))// in lib.rs
                    .get_matches();// in main.rs

    if matches.is_present("recover") {// in main.rs
        // Create a Light Client Config in an attempt to recover the file.// in lib.rs
        let config = LightClientConfig {// in lib.rs
            server: "0.0.0.0:0".parse().unwrap(),// in lib.rs
            chain_name: "main".to_string(),// in lib.rs
            sapling_activation_height: 0,// in lib.rs
            consensus_branch_id: "000000".to_string(),// in lib.rs
            anchor_offset: 0,// in lib.rs
            no_cert_verification: false,// in lib.rs
            data_dir: None,// in lib.rs
        };// in lib.rs

        match LightClient::attempt_recover_seed(&config) {// in lib.rs
            Ok(seed) => println!("Recovered seed: '{}'", seed),// in lib.rs
            Err(e)   => eprintln!("Failed to recover seed. Error: {}", e)// in lib.rs
        };// in lib.rs
        return;
    }

    let command = matches.value_of("COMMAND");// in main.rs
    let params = matches.values_of("PARAMS").map(|v| v.collect()).or(Some(vec![])).unwrap();// in main.rs

    let maybe_server   = matches.value_of("server").map(|s| s.to_string());// in main.rs

    let seed           = matches.value_of("seed").map(|s| s.to_string());// in main.rs
    let maybe_birthday = matches.value_of("birthday");// in main.rs
    
    if seed.is_some() && maybe_birthday.is_none() {// in main.rs
        eprintln!("ERROR!");// in main.rs
        eprintln!("Please specify the wallet birthday (eg. '--birthday 600000') to restore from seed.");// in main.rs
        eprintln!("This should be the block height where the wallet was created. If you don't remember the block height, you can pass '--birthday 0' to scan from the start of the blockchain.");// in main.rs
        return;// in main.rs
    }// in main.rs

    let birthday = match maybe_birthday.unwrap_or("0").parse::<u64>() {// in main.rs
                        Ok(b) => b,// in main.rs
                        Err(e) => {// in main.rs
                            eprintln!("Couldn't parse birthday. This should be a block number. Error={}", e);// in main.rs
                            return;// in main.rs
                        }// in main.rs
                    };// in main.rs

    let server = LightClientConfig::get_server_or_default(maybe_server);// in main.rs

    // Test to make sure the server has all of scheme, host and port// in main.rs
    if server.scheme_str().is_none() || server.host().is_none() || server.port_part().is_none() {// in main.rs
        eprintln!("Please provide the --server parameter as [scheme]://[host]:[port].\nYou provided: {}", server);// in main.rs
        return;// in main.rs
    }

    let dangerous = matches.is_present("dangerous");// in main.rs
    let nosync = matches.is_present("nosync");// in main.rs
    let (command_tx, resp_rx) = match startup(server, dangerous, seed, birthday, !nosync, command.is_none()) {// in main.rs
        Ok(c) => c,// in main.rs
        Err(e) => {// in main.rs
            eprintln!("Error during startup: {}", e);// in main.rs
            error!("Error during startup: {}", e);// in main.rs
            match e.raw_os_error() {// in main.rs
                Some(13) => {// in main.rs
                    startup_helpers::report_permission_error();// in main.rs
                },// in main.rs
                _ => {}// in main.rs
            }// in main.rs
            return;// in main.rs
        }// in main.rs
    };// in main.rs

    if command.is_none() {// in main.rs
        start_interactive(command_tx, resp_rx);// in main.rs
    } else {// in main.rs
        command_tx.send(// in main.rs
            (command.unwrap().to_string(),// in main.rs
                params.iter().map(|s| s.to_string()).collect::<Vec<String>>()))// in main.rs
            .unwrap();// in main.rs

        match resp_rx.recv() {// in main.rs
            Ok(s) => println!("{}", s),// in main.rs
            Err(e) => {// in main.rs
                let e = format!("Error executing command {}: {}", command.unwrap(), e);// in main.rs
                eprintln!("{}", e);// in main.rs
                error!("{}", e);// in main.rs
            }// in main.rs
        }// in main.rs

        // Save before exit// in main.rs
        command_tx.send(("save".to_string(), vec![])).unwrap();// in main.rs
        resp_rx.recv().unwrap();// in main.rs
    }// in main.rs
}// in main.rs

fn startup(server: http::Uri, dangerous: bool, seed: Option<String>, birthday: u64, first_sync: bool, print_updates: bool)  // in lib.rs
        -> io::Result<(Sender<(String, Vec<String>)>, Receiver<String>)> {  // in lib.rs
    // Try to get the configuration// in lib.rs
    let (config, latest_block_height) = LightClientConfig::create(server.clone(), dangerous)?;  // in lib.rs

    // Configure logging first. // in lib.rs
    let log_config = get_log_config(&config)?;  // in lib.rs
    log4rs::init_config(log_config).map_err(|e| {// in lib.rs
        std::io::Error::new(ErrorKind::Other, e)// in lib.rs
    })?;// in lib.rs

    let lightclient = match seed {// in lib.rs
        Some(phrase) => Arc::new(LightClient::new_from_phrase(phrase, &config, birthday)?),// in lib.rs
        None => {// in lib.rs
            if config.wallet_exists() {// in lib.rs
                Arc::new(LightClient::read_from_disk(&config)?)// in lib.rs
            } else {// in lib.rs// in lib.rs
                println!("Creating a new wallet");// in lib.rs// in lib.rs
                Arc::new(LightClient::new(&config, latest_block_height)?)// in lib.rs
            }// in lib.rs
        }// in lib.rs
    };// in lib.rs

    // Print startup Messages// in lib.rs
    info!(""); // Blank line// in lib.rs
    info!("Starting Zecwallet-CLI");// in lib.rs
    info!("Light Client config {:?}", config);// in lib.rs

    if print_updates {// in lib.rs
        println!("Lightclient connecting to {}", config.server);// in lib.rs
    }// in lib.rs

    // At startup, run a sync.// in lib.rs
    if first_sync {// in lib.rs
        let update = lightclient.do_sync(true);// in lib.rs
        if print_updates {// in lib.rs
            match update {// in lib.rs after addition!
                Ok(j) => {// in lib.rs after addition!
                    println!("{}", j.pretty(2));// in lib.rs after addition!
                },// in lib.rs after addition!
                Err(e) => println!("{}", e)// in lib.rs after addition!
            }// in lib.rs after addition!
        }// in lib.rs
    }// in lib.rs

    // Start the command loop// in lib.rs
    let (command_tx, resp_rx) = command_loop(lightclient.clone());// in lib.rs

    Ok((command_tx, resp_rx))// in lib.rs
}


fn start_interactive(command_tx: Sender<(String, Vec<String>)>, resp_rx: Receiver<String>) {// in lib.rs
    // `()` can be used when no completer is required// in lib.rs
    let mut rl = rustyline::Editor::<()>::new();// in lib.rs

    println!("Ready!");// in lib.rs

    let send_command = |cmd: String, args: Vec<String>| -> String {// in lib.rs
        command_tx.send((cmd.clone(), args)).unwrap();// in lib.rs
        match resp_rx.recv() {// in lib.rs
            Ok(s) => s,// in lib.rs
            Err(e) => {// in lib.rs
                let e = format!("Error executing command {}: {}", cmd, e);// in lib.rs
                eprintln!("{}", e);// in lib.rs
                error!("{}", e);// in lib.rs
                return "".to_string()// in lib.rs
            }// in lib.rs
        }// in lib.rs
    };// in lib.rs

    let info = &send_command("info".to_string(), vec![]);// in lib.rs
    let chain_name = json::parse(info).unwrap()["chain_name"].as_str().unwrap().to_string();// in lib.rs

    loop {// in lib.rs
        // Read the height first// in lib.rs
        let height = json::parse(&send_command("height".to_string(), vec![])).unwrap()["height"].as_i64().unwrap();// in lib.rs

        let readline = rl.readline(&format!("({}) Block:{} (type 'help') >> ",// in lib.rs
                                                    chain_name, height));// in lib.rs
        match readline {// in lib.rs
            Ok(line) => {// in lib.rs
                rl.add_history_entry(line.as_str());// in lib.rs
                // Parse command line arguments// in lib.rs
                let mut cmd_args = match shellwords::split(&line) {// in lib.rs
                    Ok(args) => args,// in lib.rs
                    Err(_)   => {// in lib.rs
                        println!("Mismatched Quotes");// in lib.rs
                        continue;// in lib.rs
                    }// in lib.rs
                };// in lib.rs

                if cmd_args.is_empty() {// in lib.rs
                    continue;// in lib.rs
                }// in lib.rs

                let cmd = cmd_args.remove(0);// in lib.rs
                let args: Vec<String> = cmd_args;// in lib.rs

                println!("{}", send_command(cmd, args));// in lib.rs

                // Special check for Quit command.// in lib.rs
                if line == "quit" {// in lib.rs
                    break;// in lib.rs
                }// in lib.rs
            },// in lib.rs
            Err(rustyline::error::ReadlineError::Interrupted) => {// in lib.rs
                println!("CTRL-C");// in lib.rs
                info!("CTRL-C");// in lib.rs
                println!("{}", send_command("save".to_string(), vec![]));// in lib.rs
                break// in lib.rs
            },// in lib.rs
            Err(rustyline::error::ReadlineError::Eof) => {// in lib.rs
                println!("CTRL-D");// in lib.rs
                info!("CTRL-D");// in lib.rs
                println!("{}", send_command("save".to_string(), vec![]));// in lib.rs
                break// in lib.rs
            },// in lib.rs
            Err(err) => {// in lib.rs
                println!("Error: {:?}", err);// in lib.rs
                break// in lib.rs
            }// in lib.rs
        }// in lib.rs
    }// in lib.rs
}// in lib.rs


fn command_loop(lightclient: Arc<LightClient>) -> (Sender<(String, Vec<String>)>, Receiver<String>) {// in lib.rs
    let (command_tx, command_rx) = channel::<(String, Vec<String>)>();// in lib.rs
    let (resp_tx, resp_rx) = channel::<String>();// in lib.rs

    let lc = lightclient.clone();// in lib.rs
    std::thread::spawn(move || {// in lib.rs
        loop {// in lib.rs
            match command_rx.recv_timeout(std::time::Duration::from_secs(5 * 60)) {// in lib.rs
                Ok((cmd, args)) => {// in lib.rs
                    let args = args.iter().map(|s| s.as_ref()).collect();// in lib.rs

                    let cmd_response = commands::do_user_command(&cmd, &args, lc.as_ref());// in lib.rs
                    resp_tx.send(cmd_response).unwrap();// in lib.rs

                    if cmd == "quit" {// in lib.rs
                        info!("Quit");// in lib.rs
                        break;// in lib.rs
                    }// in lib.rs
                },// in lib.rs
                Err(_) => {// in lib.rs
                    // Timeout. Do a sync to keep the wallet up-to-date. False to whether to print updates on the console// in lib.rs
                    info!("Timeout, doing a sync");// in lib.rs
                    match lc.do_sync(false) {// in lib.rs added
                        Ok(_) => {},// in lib.rs added
                        Err(e) => {error!("{}", e)}// in lib.rs added
                    }// in lib.rs added
                    
                }// in lib.rs
            }// in lib.rs
        }// in lib.rs
    });// in lib.rs

    (command_tx, resp_rx)// in lib.rs
}// in lib.rs
