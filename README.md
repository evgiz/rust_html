# rust_html

The minimal rust html template library

## About

**rust_html** is a tiny templating library that let's you easily create
reusable HTML templates and components:

```rust
use rust_html::{rhtml, Template};

let card_component = |title: &str| {
    rhtml! { r#"
        <div class="card">
            {title}
        </div>
    "#}
};

let title = "My Website";
let my_template: Template = rhtml! {r#"
    <div class="page">
        <h1>{title}</h1>
        {card_component("Card A")}
        {card_component("Card B")}
        {card_component("Card C")}
    </div>
"#};

let html_string: String = my_template.into();
println!("{}", html_string);

```

### Why use **rust_html**?

- Valid HTML syntax is enforced at compile-time
- Runtime rust values are automatically escaped to protect against injection attacks
- You can inject any expression or literal (not just identifiers)

The library is designed for creating reusable components for SSR
(server-side rendering), and is particularly nice in combination with front end libraries
like `alpine.js` or `htmx`. Unlike some other templating libraries, you can use the
standard HTML syntax directly and keep the templates next to your other Rust code.

## Installation

Inside your project directory, add using cargo:

```bash
cargo add rust_html
```

## Usage

### Types

The library has only 5 exported functions/types:

- `rhtml!`: The main macro for creating templates
- `Template`: represents a reusable HTML template
- `Render`: trait for implementing reusable `struct` components
- `Unescaped`: string wrapper for inserting unescaped values
- `TemplateGroup`: wrapper to insert a `Vec<Template>`

### The `rhtml!` macro

The `rhtml!` macro accepts a single string literal as input, typically
`"my text here"` or `r#"my text here"#` which is a bit more convenient for HTML.
The macro returns a `Template` struct that ensures injection safety when
reusing template within templates.

Inside the macro string you can inject anything that implements either the
`std::fmt::Display` or `::Render` trait by using brackets `{}`.
You can escape brackets by using two of them in a row (`{{` or `}}`).

> [!NOTE]  
> The `Template` struct itself does not implement the `Display` trait.
> To print or return the HTML value as a `String` you can use `String::from(my_template)`
> or just `my_template.into()` where applicable.

### Example - Reusable Components

```rust
use rust_html::{rhtml, Template, TemplateGroup};

/// Reusable card component with a title property
fn card_component(title: &str) -> Template {
    rhtml! { r#"<div class="card">{title}</div>"# }
}

/// Reusable card group component that creates N cards
fn card_row_component(n_cards: u32, container_class: &str) -> Template {
    // For injecting lists of templates, we can use a TemplateGroup
    let cards: TemplateGroup = (0..n_cards)
        .map(|card_index| {
            let title = format!("Card {}", card_index);
            card_component(&title)
        })
        .collect();
    rhtml! { r#"
        <div class="{container_class}">
            {cards}
        </div>
    "# }
}

// Server endpoint
fn your_endpoint() -> String {
    let page_template: Template = rhtml! { r#"
        <div class="page">
            {card_row_component(3, "my_card_row")}
        </div>
    "# };
    // Convert the `Template` to `String`
    // This is typically only done in the endpoint just before
    // returning the full HTML. Make sure you also return a
    // `Content-Type` of `text/html` in your response
    page_template.into()
}
```

The `your_endpoint` function will return the following HTML:

```html
<div class="page">
  <div class="my_card_row">
    <div class="card">Card 0</div>
    <div class="card">Card 1</div>
    <div class="card">Card 2</div>
  </div>
</div>
```

</details>

### Expressions inside a template

In some cases you might want to include simple logic in your
template directly. You can use any valid Rust expression inside the macro:

```rust
use rust_html::rhtml;

fn main() {
    let age = 32;
    let page = rhtml! { r#"
        <div>
            Bob is {if age >= 18 { "an adult" } else { "not an adult" }}
        </div>
    "# };
    println!("{}", String::from(page));
    // Output is '<div>Bob is an adult</div>'
}
```

### Structs as reusable components

You can also use structs as components by implementing the `Render` trait.

```rust
use rust_html::{rhtml, Render, Template};

// Components must derive Clone
#[derive(Clone)]
struct CardComponent {
    title: String,
    content: String,
}

// Implement rust_html rendering for our component
impl Render for CardComponent {
    fn render(&self) -> Template {
        rhtml! {r#"
            <div class="card">
                <h1>{self.title}</h1>
                <p>{self.content}</p>
            </div>
        "#}
    }
}

fn main() {
    let my_card = CardComponent {
        title: "Welcome".to_string(),
        content: "This is a card".to_string(),
    };
    let page = rhtml! {r#"
        <div class="page">
            {my_card}
        </div>
    "#};
}
```

## Escaping

Template input is escaped by default to prevent injection attacks, for instance if
a user were to register with a name that contains a `<script>` tag.
The following snippet:

```rust
let sketchy_user_input = "<script>alert('hi')</script>";
let page = rhtml! {r#"<div>{sketchy_user_input}</div>"#};
println!("{}", String::from(page));
```

Generates a string where dangerous characters are escaped:

```html
<div>&lt;script&gt;alert(&#x27;hi&#x27;)&lt;&#x2F;script&gt;</div>
```

### Unescaping

If you need the unescaped value, you can use the `Unescaped` wrapper.

> [!CAUTION]  
> Never use `Unescaped` on untrusted user input or if you don't
> know what you're doing.

```rust
use rust_html::{rhtml, Unescaped};
let sketchy_user_input = "<script>alert('hi')</script>";
let unescaped = Unescaped(sketchy_user_input.to_string());
let page = rhtml! {r#"<div>{unescaped}</div>"#};
println!("{}", String::from(page));
```

...which results in this string:

```html
<div>
  <script>
    alert("hi");
  </script>
</div>
```

## Integration with web frameworks

Integrating with any web framework is trivial - simply convert the
template string to the response type for the given framework.
If you're using Axum you can add the `axum` feature to get support
for their `IntoResponse` trait.

## Related projects

- [maud](https://github.com/lambda-fairy/maud): rust syntax for HTML
- [askama](https://github.com/djc/askama): jinja like templating library
- [tera](https://github.com/Keats/tera): jinja2 like templating library
- [handlebars-rust](https://github.com/sunng87/handlebars-rust): handlebars templating language for rust

Look at the [AWWY](https://www.arewewebyet.org/topics/templating/) website for more examples.

## Contributing

Run tests:

```bash
cargo test -p rust_html_tests
```

Run doc tests:

```bash
cargo test -p rust_html_macros
```
