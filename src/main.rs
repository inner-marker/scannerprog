mod components;
mod database_view;
mod scanner_db;
mod scanner_interaction;

fn main() {
    dioxus::launch(components::App);
}
