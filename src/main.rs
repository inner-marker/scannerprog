use dioxus::desktop::tao::dpi::{LogicalSize}; // tao's DPI types, re-exported through dioxus
use dioxus::desktop::{Config, WindowBuilder};       // desktop-specific config + window handle hook
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

mod database_view;
mod scanner_db;
mod scanner_interaction;
mod components;
mod messages;


fn main() {
    let window = WindowBuilder::new()
        .with_title("Scanner Programmer")                  // title bar text
        .with_inner_size(LogicalSize::new(1500.0, 1000.0))  // content area size in logical (DPI-scaled) pixels
        .with_visible(true);                              // start hidden so the user never sees it at the default spot

    // Launch the desktop renderer with our window config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window))
        .launch(components::App);


    // dioxus::launch(components::App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        database_view::DatabaseView {}
    }
}
