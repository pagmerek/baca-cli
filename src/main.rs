#[macro_use]
extern crate clap;
use crate::baca::details::Language;
use crate::workspace::TaskConfig;
use clap::{App, AppSettings};
use colored::Colorize;
use std::path::Path;
use std::str::FromStr;
use tracing::Level;

mod baca;
mod command;
mod log;
mod model;
mod parse;
mod workspace;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml).setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();

    let log_level = match matches.occurrences_of("verbose") {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    log::init_logging(log_level);

    if let Some(matches) = matches.subcommand_matches("init") {
        let host = matches.value_of("host").unwrap();
        let login = matches.value_of("login").unwrap();
        let password = matches.value_of("password").unwrap();

        tracing::info!("Using BaCa host: {}", host);
        tracing::info!("Using BaCa login: {}", login);
        tracing::info!("Using BaCa password: {}", password);

        command::init(host, login, password);
        return; // todo: some error handling
    }

    // todo: print test logs as well
    if let Some(matches) = matches.subcommand_matches("details") {
        let submit_id = matches.value_of("id").unwrap();
        tracing::info!("Printing details for submit: {}", submit_id);

        command::details(submit_id);
        return;
    }

    if matches.subcommand_matches("refresh").is_some() {
        println!("Refreshing BaCa session...");
        command::refresh();
        return;
    }

    // if task is configured, filter logs, add --all switch
    if let Some(matches) = matches.subcommand_matches("log") {
        let last_n = matches.value_of("amount").unwrap().parse().unwrap();
        command::log(last_n);
        return;
    }

    if matches.subcommand_matches("tasks").is_some() {
        command::tasks();
        return;
    }

    if let Some(matches) = matches.subcommand_matches("submit") {
        if matches.subcommand_matches("clear").is_some() {
            println!("Clearing task config...");
            workspace::remove_task();
            return;
        }

        let task_id = matches.value_of("task_id");
        let file_path = matches.value_of("file");
        let to_zip = matches.is_present("zip");
        let lang = matches.value_of("language");
        let saved = workspace::read_task();

        if let Some(lang) = lang {
            if let Ok(Language::Unsupported) = Language::from_str(lang) {
                println!("{} {}", lang, "is not yet supported!! Please create an issue at https://github.com/hjaremko/baca-cli/issues".bright_red());
                return;
            }
        }

        if saved.is_none() {
            tracing::info!("Task not loaded.");
        }

        if task_id.is_none() && saved.is_none() {
            println!(
                "{}",
                "Please provide task_id. Type 'baca submit -h' for more info.".bright_red()
            );
            return;
        }

        if file_path.is_none() && saved.is_none() {
            println!(
                "{}",
                "Please provide file. Type 'baca submit -h' for more info.".bright_red()
            );
            return;
        }

        if lang.is_none() && saved.is_none() {
            println!(
                "{}",
                "Please provide language. Type 'baca submit -h' for more info.".bright_red()
            );
            return;
        }

        let saved = saved.unwrap_or(TaskConfig {
            id: "".to_string(),
            file: "".to_string(),
            to_zip: false,
            language: Language::Unsupported,
        });

        let task_id = match task_id {
            None => saved.id.clone(),
            Some(id) => id.to_string(),
        };

        let file_path = match file_path {
            None => saved.file.clone(),
            Some(file) => file.to_string(),
        };

        let lang = match lang {
            None => saved.language,
            Some(lang) => Language::from_str(lang).unwrap(),
        };

        let to_zip = match to_zip {
            true => true,
            false => saved.to_zip,
        };

        if matches.is_present("default") {
            workspace::save_task(&task_id, &file_path, to_zip, lang);
        }

        let file_to_submit = if to_zip {
            let path = Path::new(&file_path);
            let res = workspace::zip_file(path);

            if let Err(e) = res {
                println!(
                    "Error zipping {}! Error: {}",
                    path.to_str().unwrap(),
                    e.to_string().bright_red()
                );
                return;
            }

            res.unwrap()
        } else {
            file_path
        };

        command::submit(&task_id, file_to_submit.as_str(), &lang);
        return;
    }
}
