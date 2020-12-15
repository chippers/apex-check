// Copyright 2020 Chip Reed
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::env::args;
use std::io::{self, Write};
use std::process::exit;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn main() {
    // ensure that we only have a single argument
    let args = args();
    if args.len() != 2 {
        eprintln!("Usage: apex-check <APEX_DOMAIN>");
        eprintln!("Only a single argument is supported, see --help for more information.");
        exit(1);
    }

    // check for our only flag, the help command
    let arg = args.last().unwrap();
    if arg == "-h" || arg == "--help" {
        println!("Apex Check v{}", env!("CARGO_PKG_VERSION"));
        println!("{}", env!("CARGO_PKG_AUTHORS").replace(":", ", "));
        println!("{}", env!("CARGO_PKG_DESCRIPTION"));
        println!("Useful for sites on GitHub Pages where 1 of the 4 will fail by default.");
        println!("\nUSAGE:\n\tapex-check <APEX_DOMAIN>");
        println!("\nFLAGS:\n\t-h, --help\tPrints this message");
        println!("\nEXAMPLE:\n\tapex-check example.org");
        exit(0);
    }

    // otherwise check that the input is valid
    if arg.starts_with("http://") || arg.starts_with("https://") {
        eprintln!("The input url provided should not have a scheme (e.g. http://)");
        exit(1);
    }

    if arg.contains('/') {
        eprintln!("The input url should just be the domain name, no paths");
        exit(1);
    }

    // keep track of any errors
    let mut had_error = false;

    // special version of stdout for color support
    let mut stdout = StandardStream::stdout(if atty::is(atty::Stream::Stdout) {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });

    // check the http requests for any errors and print the resulting reports
    for url in vec![
        format!("http://{}", arg),
        format!("http://www.{}", arg),
        format!("https://{}", arg),
        format!("https://www.{}", arg),
    ] {
        match ureq::get(&url).call().into_synthetic_error() {
            None => write_report(&mut stdout, &url, true),
            Some(error) => {
                had_error = true;
                write_report(&mut stdout, &url, false);
                eprintln!("{:#?}", error);
            }
        }
    }

    // reset all styles before exiting to be nice
    stdout
        .reset()
        .expect("unable to reset terminal color styles");

    if had_error {
        exit(1);
    }
}

/// Write a standard format report based on the passed information
fn write_report(stdout: &mut StandardStream, url: &str, ok: bool) {
    // get the styles based on if the request was ok
    let (color, status) = if ok {
        (Color::Green, "✓")
    } else {
        (Color::Red, "X")
    };

    // capture any stdout errors
    || -> io::Result<()> {
        stdout.set_color(ColorSpec::new().set_fg(Some(color)))?;
        writeln!(stdout, "{} — {}", status, url)
    }()
    .expect("unable to write report to stdout")
}
