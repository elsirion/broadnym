use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Link};

fn main() {
    // set up logging
    tracing_wasm::set_as_global_default();
    console_error_panic_hook::set_once();
    provide_meta_context();

    mount_to_body(move || {
        view! {
           <body class="dark:bg-gray-900">
               <h1>Hello World</h1>
           </body>
        }
    })
}
