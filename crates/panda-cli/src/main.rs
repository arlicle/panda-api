use std::ffi::OsStr;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let mut arguments = std::env::args_os().skip(1);

    match (arguments.next(), arguments.next()) {
        (Some(argument), None)
            if argument == OsStr::new("--version") || argument == OsStr::new("-V") =>
        {
            println!("{VERSION}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!(
                "Panda API {VERSION} is at M0 architecture foundation; commands are not implemented"
            );
            ExitCode::from(2)
        }
    }
}
