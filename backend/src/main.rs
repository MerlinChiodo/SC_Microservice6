use moon::*;

async fn frontend() -> Frontend {
    Frontend::new()
        .title("MZ Demo")
        .default_styles(false)
        .append_to_head(r#"<link rel="stylesheet" href="https://unpkg.com/@picocss/pico@latest/css/pico.min.css">"#)
        .append_to_head(r#"<link rel="stylesheet" href="/_api/public/custom.css">"#)
        .body_content(r#"<div id="app"></div>"#)
}


async fn up_msg_handler(_: UpMsgRequest<()>) {}

#[moon::main]
async fn main() -> std::io::Result<()> {
    start(frontend, up_msg_handler, |_| {}).await
}
