use dioxus::desktop::tao::dpi::{LogicalSize}; // tao's DPI types, re-exported through dioxus
use dioxus::desktop::{Config, WindowBuilder};       // desktop-specific config + window handle hook
use dioxus::prelude::*;

/// Application stylesheet loaded by the root component.
const MAIN_CSS: Asset = asset!("/assets/main.css");

mod database;
mod database_view;
mod models;
mod scanner_interaction;
mod components;
mod messages;
mod confirm;


/// Starts the desktop app with a sized, titled window and no native menu.
fn main() {
    let window = WindowBuilder::new()
        .with_title("Scanner Programmer")                  // title bar text
        .with_inner_size(LogicalSize::new(1500.0, 1000.0))  // content area size in logical (DPI-scaled) pixels
        .with_visible(true);                              // Show the window as soon as the app starts

    // Launch the desktop renderer with our window config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window).with_menu(None))
        .launch(components::App);


    // dioxus::launch(components::App);
}

/// Root component used by the desktop launcher.
#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        database_view::DatabaseView {}
    }
}
