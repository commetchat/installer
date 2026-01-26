#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate web_view;
pub mod command;
pub mod config;

mod cli;

use std::thread;

use clap::Parser;
use log::{error, info};
use simple_logger::SimpleLogger;
use urlencoding::encode;
use web_view::*;

use crate::{cli::Args, config::WINDOW_TITLE};

fn main() {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();

    let args = Args::try_parse();
    let args = match args {
        Ok(args) => args,
        Err(err) => {
            error!("{}", err.render());
            return;
        }
    };

    log::info!("{:?}", args);

    let content = include_str!("../ui/dist/index.html");

    let mut prompt = "Install Commet";

    if args.command == "update" {
        prompt = "Update Commet";
    }

    let content = content.replace("${BUTTON_PROMPT}", prompt);

    web_view::builder()
        .title(WINDOW_TITLE)
        .content(Content::Html(content))
        .size(250, 250)
        .frameless(true)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            let handle = webview.handle();
            if arg == "start" {
                info!("Starting installation");

                let args = args.clone();
                thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_time()
                        .enable_io()
                        .build()
                        .unwrap();
                    info!("Spawning tokio runtime");

                    let set_text = |text: &str| {
                        let safe = encode(text);
                        let result = format!("setText('{}');", safe);

                        let _ = handle
                            .dispatch(move |f| {
                                f.eval(result.clone().as_str()).unwrap();
                                Ok(())
                            })
                            .unwrap();
                        ();
                    };

                    rt.block_on(async {
                        let result = match args.command.as_str() {
                            "install" => command::installer::install(args, set_text).await,
                            "update" => command::installer::install(args, set_text).await,
                            _ => {
                                set_text("Unknown Command");
                                false
                            }
                        };

                        if result {
                            let _ = handle.dispatch(|f| {
                                f.exit();
                                Ok(())
                            });
                        }

                        return;
                    });

                    info!("Finished task, shutting down");
                });
            }

            Ok(())
        })
        .run()
        .unwrap();
}
