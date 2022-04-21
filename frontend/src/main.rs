use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let page = html! {
        <body>
            <main class ="container">
            <h1>{"Heya!"}</h1>
            <article>{"We're working"}</article>
            </main>
        </body>
    };
    page
}

fn main() {
    yew::start_app::<App>();
}