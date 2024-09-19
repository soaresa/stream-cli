
use std::{
  env,
  process::exit
};

/// Prompts user to type mnemonic securely.
// TODO: decide what the return of the function is
pub fn get_account_from_prompt(venue_name: &str) -> anyhow::Result<String> {
    println!("Enter your {} mnemonic:", venue_name);

    let test_env_mnem = env::var("TSMNEM");
    // if we are in debugging or CI mode
    let mnem = match test_env_mnem.is_ok() {
        true => {
            println!("Debugging mode, using mnemonic from env variable, $TSMNEM");
            test_env_mnem.unwrap().trim().to_string()
        }
        false => match rpassword::read_password_from_tty(Some("\u{1F511} ")) {
            Ok(read) => read.trim().to_owned(),
            Err(e) => {
                println!(
                    "ERROR: could not read mnemonic from prompt, message: {}",
                    &e.to_string()
                );
                exit(1);
            }
        },
    };

    Ok(mnem)
}
