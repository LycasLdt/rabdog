use console::style;
use std::fmt::Display;

pub fn log_with_progress<K: Display, M: Display>(kind: K, message: M) {
    println!("{} - {}", style(kind).dim(), message)
}

pub fn log_error<M: Display>(message: M) {
    eprintln!("{}: {}", style("error").red().bright(), message)
}
