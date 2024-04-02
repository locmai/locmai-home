use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::footer::*;
use crate::components::header::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/locmai_home.css"/>
        <Router>
          <div class="fixed inset-0 flex justify-center sm:px-8">
            <div class="flex w-full max-w-7xl lg:px-8">
              <div class="w-full bg-white ring-1 ring-zinc-100 dark:bg-zinc-900 dark:ring-zinc-300/20" />
            </div>
          </div>
          <div class="relative flex w-full flex-col">
            <Header />
            <main class="flex-auto">
                <Routes>
                    placeholder
                </Routes>
            </main>
            <Footer/>
          </div>
        </Router>
    }
}
