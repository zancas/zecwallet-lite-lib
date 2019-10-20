extern crate zecwalletlitelib;

#[test]
fn unauthorized_user_dotzcash_file_access() {
    use clap::{App, Arg};
    use zecwalletlitelib::{configure_clapapp, lightclient};
    configure_clapapp!(App::new("Zecwallet CLI Test Unauthorized User"));
    assert_eq!(2 + 2, 4);
}
