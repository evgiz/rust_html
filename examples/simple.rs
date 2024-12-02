use rust_html::*;

fn card(title: &str, content: impl Render) -> Template {
    rhtml! { r#"
        <div class="card">
            <h2>{title}</h2>
            {content}
        </div>
    "#}
}

fn card_group() -> Template {
    let card_1 = card("Hello, rust_html", "Content here!");
    let card_2 = card("Another card", "And more content!");
    rhtml! { r#"
        <div class="container">
            <h1>Welcome to rust_html</h1>
            {card_1}
            {card_2}
        </div>
    "#}
}

fn layout(title: &str, content: impl Render) -> Template {
    rhtml! { r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>{title}</title>
            </head>
            <body>
                {content}
            </body>
        </html>
    "#}
}

fn main() {
    let card_group = card_group();
    let html_template = layout("rust_html", card_group);
    let html_string: String = html_template.into();
    println!("{}", html_string)
}
