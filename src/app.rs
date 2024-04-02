use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::header::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Header/>
            <p> Hello </p>
        </Router>
    }
}
