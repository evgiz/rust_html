use axum::{response::IntoResponse, routing::get, Router};
use rand::Rng;

use rust_html::*;

/// A fully fledged front-end calculator using
/// rust_html, alpine.js, math.js and axum to serve.
///
/// This example is just to illustrate more how you can use rust_html
///
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");
    axum::serve(listener, app)
        .await
        .expect("Failed to start axum serve");
}

/// Component for most buttons (inserting values)
fn button(symbol: &str, class: &str) -> Template {
    rhtml! { r#"
        <button class="{class}" @click="insert('{symbol}')">
            {symbol}
        </button>
    "# }
}

fn calculator() -> Template {
    let top_row = rhtml! { r#"
        <button class="clear" @click="clear">
            AC
        </button>
        {button("(", "operator")}
        {button(")", "operator")}
        {button("/", "operator")}
    "#};

    // Template group for the main buttons
    // TemplateGroup is just a wrapper for Vec<Template>
    #[rustfmt::skip]
    let main_buttons: TemplateGroup = [
        "7", "8", "9", "*",
        "4", "5", "6", "-",
        "1", "2", "3", "+"
    ]
        .into_iter()
        .enumerate()
        .map(|(i, symbol)| {
            let class = if i % 4 == 3 { "operator" } else { "" };
            button(symbol, class)
        })
        .collect();

    let bottom_row = rhtml! { r#"
        <button class="clear" @click="del">
            DEL
        </button>
        {button("0", "")}
        {button(".", "operator")}
        <button class="exec" @click="eval">
            =
        </button>
    "#};

    // Since we trust our javascript we feel safe to use Unescaped!
    let trusted_javascript = Unescaped(CALCULATOR_JS.to_string());

    // The main html component
    rhtml! { r#"
        <div class="calculator" x-data="{trusted_javascript}">
            <h2 x-text="value" class="display">
                &nbsp;
            </h2>
            <div class="grid">
                {top_row}
                {main_buttons}
                {bottom_row}
            </div>
        </div>
    "# }
}

/// The main endpoint for our calculator app
async fn root() -> impl IntoResponse {
    let stylesheet = Unescaped(CSS_STYLE.to_string());
    let page = rhtml! {r#"
        <!DOCTYPE html>
        <html>
            <head>
                <title>rust_html calculator</title>
                <script src="//unpkg.com/alpinejs" defer></script>
                <script src="//unpkg.com/mathjs"></script>
                <style>
                    {stylesheet}
                </style>
            </head>
            <body>
                <h1>Calculator</h1>
                <h2>rust_html + alpine.js</h2>
                {calculator()}
            </body>
        </html>
    "#};
    page
}

// -------- Javascript below ----------

const CALCULATOR_JS: &'static str = r#"
{
    value: '',
    insert(value) {
        if(this.value == 'ERROR') {
            this.value = value;
        } else {
            this.value += value;
        }
    },
    clear() {
        this.value = '';
    },
    del() {
        this.value = this.value.slice(0, this.value.length - 1);
    },
    eval() {
        try {
            this.value = math.evaluate(this.value);
        } catch(e) {
            this.value = 'ERROR';
        }
    }
}
"#;

// -------- Stylesheet below ----------

const CSS_STYLE: &'static str = r#"

html {
    text-align: center;
    font-family: monospace;
    padding-top: 2em;
}

h1, h2 {
    margin: 0;
    padding: 0;
    padding-bottom: .5em;
}

.calculator {
    display: inline-block;
    padding: 1em;
    border: 1px solid black;
    background: #444;
    color: white;
    border-radius: 1em;
    margin-top: 2em;
}

.display {
    height: 3em;
    line-height: 3em;
    background: #aaa;
    color: #111;
    text-align: right;
    padding: 0 1em;
    border-radius: .5em;
    margin: .5em .3em;
}

.grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
}

button {
    all: unset;
    width: 4em;
    height: 4em;
    margin: .3em;
    background: #555;
    font-size: 1.25em;
    cursor: pointer;
    border-radius: .5em;
}

button.operator {
    background: #333;
}

button.exec {
    background: orange;
}

button.clear {
    background: orangered;
}

button:hover {
    filter: brightness(110%);
}

button:active {
    filter: brightness(95%);
}

"#;
