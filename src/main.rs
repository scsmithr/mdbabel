use clap::{App, Arg};
use std::fs::File;

mod executor;
mod markdown;

fn main() {
    let matches = App::new("mdbabel")
        .version("0.1.0")
        .about("Execute markdown code blocks")
        .arg(
            Arg::with_name("INPUT_FILE")
                .help("Markdown file to read.")
                .index(1)
                .required(true),
        )
        .get_matches();

    let file_path = matches.value_of("INPUT_FILE").unwrap();
    let file = File::open(file_path).unwrap();
    let doc = markdown::Document::new(file);

    let exs = executor::Executors::default_executors();
    for directive in doc {
        match directive {
            markdown::Directive::CodeBlock { header: _, body } => {
                exs.execute(&body).unwrap();
            }
        }
    }
}
