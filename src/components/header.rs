use leptos::*;

#[component]
pub fn Header() -> impl IntoView {
    let nav_items = [("Home", "/"), ("Posts", "/posts"), ("About", "/about")];

    view! {
        <header class="pointer-events-none relative z-50 flex flex-none flex-col">
            <nav
                class="mx-auto max-w-7xl items-center text-center p-6 lg:px-8"
                aria-label="Global"
            >
             <ul class="flex rounded-full bg-white/90 px-3 text-sm font-medium text-zinc-800 shadow-lg shadow-zinc-800/5 ring-1 ring-zinc-900/5 backdrop-blur dark:bg-zinc-800/90 dark:text-zinc-200 dark:ring-white/10">
                {
                    nav_items
                        .iter()
                        .map(|(name,href)|{
                            view! {
                                <li>
                                    <a class="relative block px-3 py2 transition" href=href.to_string()> {name.to_string()} </a>
                                </li>
                            }
                        })
                        .collect_view()
                }
                <li><a href="/home" class="relative block px-3 py-2 transition"> Home </a></li>
                <li><a href="/post" class="relative block px-3 py-2 transition"> Posts </a></li>
                <li><a href="/about" class="relative block px-3 py-2 transition"> About </a></li>
             </ul>
            </nav>
        </header>
    }
}
