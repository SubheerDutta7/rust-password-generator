use std::env;
use std::process::ExitCode;

use rust_password_generator::{
    copy_passwords_to_clipboard, format_output, format_passwords_for_file, generate_passwords,
    parse_args, print_help, write_passwords_to_file, Action, APP_NAME, APP_VERSION,
};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            eprintln!("Run with --help to see usage.");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();

    match parse_args(&args)? {
        Action::Help => {
            print_help();
            Ok(())
        }
        Action::Version => {
            println!("{APP_NAME} {APP_VERSION}");
            Ok(())
        }
        Action::Generate(config) => {
            let result = generate_passwords(&config)?;
            println!("{}", format_output(&result, &config));

            if let Some(path) = &config.output_file {
                let file_content = format_passwords_for_file(&result);
                write_passwords_to_file(path, &file_content)?;
                if config.pretty {
                    eprintln!("Saved generated password(s) to {path}.");
                }
            }

            if config.copy_to_clipboard {
                copy_passwords_to_clipboard(&result.passwords)?;
                if config.pretty {
                    eprintln!("Copied generated password(s) to the clipboard.");
                }
            }

            Ok(())
        }
    }
}
