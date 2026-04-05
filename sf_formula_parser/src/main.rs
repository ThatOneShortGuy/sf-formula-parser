use std::{env, fs, process::ExitCode};

use sf_formula_parser::validate_expression_detailed;

fn main() -> ExitCode {
    let mut args = env::args();
    let program = args
        .next()
        .unwrap_or_else(|| "sf_formula_parser".to_string());

    let Some(path) = args.next() else {
        eprintln!("Usage: {program} <file.sff>");
        return ExitCode::from(2);
    };

    if args.next().is_some() {
        eprintln!("Usage: {program} <file.sff>");
        return ExitCode::from(2);
    }

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(err) => {
            eprintln!("failed to read `{path}`: {err}");
            return ExitCode::from(1);
        }
    };

    match validate_expression_detailed(&source) {
        Ok(()) => {
            println!("ok: expression is valid");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err.rendered);
            ExitCode::from(1)
        }
    }
}
