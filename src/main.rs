use clipboard::{ClipboardContext, ClipboardProvider};
use serde::Deserialize;
use std::{fmt::Write, process::exit};
use structopt::StructOpt;

/// Get most recent version of a crate
#[derive(StructOpt, Debug)]
#[structopt(name = "crate-version")]
struct Opt {
    /// Print verbose error output
    #[structopt(long = "verbose", short = "v")]
    verbose: bool,

    /// Copy the result to the system clipboard
    #[structopt(long = "copy", short = "c")]
    clipboard: bool,

    /// Name of crate
    #[structopt(name = "CRATE_NAME")]
    crate_name: String,
}

#[derive(Deserialize, Debug)]
struct ApiResponse {
    #[serde(rename(deserialize = "crate"))]
    crate_: Option<Crate>,
    #[serde(default)]
    errors: Vec<ApiError>,
}

#[derive(Deserialize, Debug)]
struct Crate {
    max_version: String,
}

#[derive(Deserialize, Debug)]
struct ApiError {
    detail: String,
}

type Result<T, E = Box<std::error::Error>> = std::result::Result<T, E>;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let crate_name = &opt.crate_name;

    let url = format!("https://crates.io/api/v1/crates/{}", crate_name);

    let resp = reqwest::get(&url)
        .unwrap_or_else(|e| opt.exit_with_msg("crates.io API request failed", Some(Box::new(e))))
        .json::<ApiResponse>()
        .unwrap_or_else(|e| {
            opt.exit_with_msg("Failed parsing crates.io API response", Some(Box::new(e)))
        });

    match resp.errors.len() {
        0 => match resp.crate_ {
            Some(crate_) => {
                let output = format!("{} = \"{}\"", crate_name, crate_.max_version);
                if opt.clipboard {
                    copy_to_clipboard(&output).unwrap_or_else(|e| {
                        opt.exit_with_msg("Error copying result to clipboard", Some(e))
                    });
                }
                println!("{}", output);
            }
            None => {
                let mut f = String::new();
                writeln!(f, "Something went wrong!")?;
                writeln!(f, "crates.io API request gave no errors and no data...")?;
                opt.exit_with_msg(&f, None)
            }
        },
        1 => {
            let mut f = String::new();
            writeln!(f, "Something went wrong!")?;
            writeln!(f, "Error: {}", resp.errors[0].detail)?;
            opt.exit_with_msg(&f, None)
        }
        _ => {
            let mut f = String::new();
            writeln!(f, "Something went wrong!")?;
            writeln!(f, "Errors:")?;
            for error in resp.errors {
                writeln!(f, "  {}", error.detail)?;
            }
            opt.exit_with_msg(&f, None)
        }
    }

    Ok(())
}

impl Opt {
    fn exit_with_msg<T>(&self, msg: &str, e: Option<Box<std::error::Error>>) -> T {
        eprintln!("{}", msg);

        if let (true, Some(e)) = (self.verbose, e) {
            eprintln!();
            eprintln!("{}", e);
        }

        exit(1)
    }
}

fn copy_to_clipboard(s: &str) -> Result<()> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    ctx.set_contents(s.to_string())?;
    Ok(())
}
