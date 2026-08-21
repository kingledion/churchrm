use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{A, Route, Router, Routes},
    ParamSegment, StaticSegment,
};

use crate::views::{ContactDetailPage, HomePage, NewContactPage};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/churchrm.css"/>
        <Title text="ChurchRM"/>
        <Router>
            <div class="app-shell">
                <header class="app-header">
                    <h1 class="app-title">"ChurchRM"</h1>
                    <p class="app-subtitle">"Parish contact directory"</p>
                </header>
                <main class="app-main">
                    <Routes fallback=NotFound>
                        <Route path=StaticSegment("") view=HomePage/>
                        <Route
                            path=(StaticSegment("contacts"), StaticSegment("new"))
                            view=NewContactPage
                        />
                        <Route
                            path=(StaticSegment("contacts"), ParamSegment("id"))
                            view=ContactDetailPage
                        />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <section class="page">
            <h2 class="page-heading">"Not found"</h2>
            <p>"That page does not exist."</p>
            <A href="/" attr:class="button button-secondary">"Go home"</A>
        </section>
    }
}
