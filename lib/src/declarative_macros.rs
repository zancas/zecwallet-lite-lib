#[macro_export]
macro_rules! configure_clapapp {
    ( $freshapp: expr ) => {
    $freshapp.version("1.0.0")
            .arg(Arg::with_name("dangerous")
                .long("dangerous")
                .help("Disable server TLS certificate verification. Use this if you're running a local lightwalletd with a self-signed certificate. WARNING: This is dangerous, don't use it with a server that is not your own.")
                .takes_value(false))
            .arg(Arg::with_name("nosync")
                .help("By default, zecwallet-cli will sync the wallet at startup. Pass --nosync to prevent the automatic sync at startup.")
                .long("nosync")
                .short("n")
                .takes_value(false))
            .arg(Arg::with_name("recover")
                .long("recover")
                .help("Attempt to recover the seed from the wallet")
                .takes_value(false))
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
            .arg(Arg::with_name("COMMAND")
                .help("Command to execute. If a command is not specified, zecwallet-cli will start in interactive mode.")
                .required(false)
                .index(1))
            .arg(Arg::with_name("PARAMS")
                .help("Params to execute command with. Run the 'help' command to get usage help.")
                .required(false)
                .multiple(true))
            .get_matches()
    };
}
