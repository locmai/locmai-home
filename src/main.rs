use app::*;
use leptos::*;
mod app;
mod pages;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    logging::log!("CSR mode - mounting to body");

    mount::mount_to_body(|| {
        view! { <App/> }
    });
}
