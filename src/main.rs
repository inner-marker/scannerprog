use dioxus::prelude::*;
const MAIN_CSS: Asset = asset!("/assets/main.css");

mod database_view;
mod scanner_db;
mod scanner_interaction;
mod components;


fn main() {
    dioxus::launch(components::App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: MAIN_CSS }
        database_view::DatabaseView {}
    }
}
