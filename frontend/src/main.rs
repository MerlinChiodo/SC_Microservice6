use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let page = html! {
        <body>
        <main class = "container">
            <article>
                <div>
                  <hgroup>
                    <h1>{"Registrieren"}</h1>
                    <h2>{"Neuen Account erstellen"}</h2>
                  </hgroup>
                  <form>
                    <div class = "grid">
                        <input type="text" name="username" placeholder="Nutzername" aria-label="Login" autocomplete="nickname"/>
                        <input type="text" name="email" placeholder="E-Mail Adresse" aria-label="Login" autocomplete="email"/>
                    </div>

                    <input type="text" name="cit_code" placeholder="Registrierungsschlüssel" aria-label="Registrierungsschlüssel"/>
                    <input type="password" name="password" placeholder="Passwort" aria-label="Password" autocomplete="current-password"/>
                    <button type="button" class="contrast">{"Bestätigen"}</button>
                  </form>
                </div>
            </article>
        </main>
        </body>
    };
    page
}

fn main() {
    yew::start_app::<App>();
}