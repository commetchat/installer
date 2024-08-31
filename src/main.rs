#![windows_subsystem = "windows"]

extern crate web_view;

use web_view::*;

fn main() {
    web_view::builder()
        .title("Commet Installer")
        .content(Content::Html(include_str!("../page/index.html")))
        .size(250, 300)
        .frameless(true)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|_webview, _arg| Ok(()))
        .run()
        .unwrap();
}
