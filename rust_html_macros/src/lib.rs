use parse::compile_check_html;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[macro_use]
extern crate html5ever;

mod parse;
mod util;

use util::*;

/// rust_html - The minimal Rust HTML templating library
///
/// The rhtml macro enables you to easily create composable HTML templates by
/// injecting any identifier, expression or literal that implements
/// the standard `std::fmt::Display` trait. The macro protects against
/// injection attacks and validates the HTML syntax in compile time.
///
///
/// ## Examples
///
/// Simple example with injected value:
///
/// ```rust
/// use rust_html::rhtml;
/// let value = "Hello, World";
/// let page = rhtml! { "<span>{value}</span>" };
/// assert_eq!(&String::from(page), "<span>Hello, World</span>");
/// ```
///
/// Example with reusable component:
///
/// ```rust
/// use rust_html::rhtml;
/// let card = |title: &str| {
///     rhtml! { r#"<div class="card">{title}</div>"# }
/// };
/// let page = rhtml! { r#"
///     <div class="container">
///         {card("Card A")}
///         {card("Card B")}
///     </div>
/// "#};
/// assert_eq!(
///     &String::from(page), r#"
///     <div class="container">
///         <div class="card">Card A</div>
///         <div class="card">Card B</div>
///     </div>
/// "#);
/// ```
///
/// Runtime rust-values are escaped by default to
/// protect against injection attacks:
///
/// ```rust
/// use rust_html::rhtml;
/// let value = "<script>";
/// let page = rhtml! { "<span>{value}</span>" };
/// assert_eq!(&String::from(page), "<span>&lt;script&gt;</span>");
/// ```
///
/// You can use raw/unescaped values with the `Unescaped`
/// wrapper, but never do this with untrusted user input!
/// Unescaped will also break the compile-time guarantee of
/// valid HTML syntax in your template.
///
/// ```rust
/// use rust_html::{rhtml, Unescaped};
/// let value = Unescaped("<script>".to_string());
/// let page = rhtml! { "<span>{value}</span>" };
/// assert_eq!(&String::from(page), "<span><script></span>");
/// ```
///
/// The macro does compile time syntax checking of your HTML.
/// The following example will not compile due to missing
/// quotes around the class name:
///
/// ```rust compile_fail
/// use rust_html::rhtml;
/// let class = "red";
/// let page = rhtml! { "<div class={class}></div>" };
/// assert_eq!(String::from(page), "<div class=my_class></div>");
/// ```
///
/// For more examples and documentation, check out the README.md
///
#[proc_macro]
pub fn rhtml(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand(input.into()).into()
}

/// Main macro implementation (using types from proc_macro2 crate)
/// Parses input, generates the string template and list of rust evaluators
/// to produce the final output TokenStream
fn expand(input: TokenStream) -> TokenStream {
    // Parse input string inside macro
    let input_string = match validate_input(input) {
        Ok(string) => string,
        Err(err) => {
            return err;
        }
    };

    // Convert contents to html template and list of rust evaluators
    let (html_parts, rust_evaluators) = match parse_rhtml(&input_string) {
        Ok(result) => result,
        Err(err) => return err,
    };

    // Compile time HTML syntax check
    let html_for_validate = trim_whitespace_per_line(&html_parts.join(""));
    if let Err(error) = compile_check_html(&html_for_validate) {
        return error;
    }

    // Build output TokenStream
    let template_parts_ident = format_ident!("template_parts");
    let mut html_literals: Vec<_> = html_parts
        .iter()
        .map(|part| string_to_literal(part))
        .collect();
    let Some(template_end_literal) = html_literals.pop() else {
        return compile_error("internal");
    };

    if html_literals.len() != rust_evaluators.len() {
        return compile_error(
            "unexpected number of parameters, this might be an internal rust_html error",
        );
    }

    quote! {
        {
            let #template_parts_ident: Vec<(&'static str, rust_html::Template)> = vec![#(
                (
                    #html_literals,
                    rust_html::Render::render(&#rust_evaluators)
                )
            ),*];
            rust_html::Template::build_internal(
                #template_parts_ident,
                #template_end_literal
            )
        }
    }
}

/// Parses rhtml content. On success returns HTML string template and list of
/// rust token streams to inject into the string.
fn parse_rhtml(input: &str) -> Result<(Vec<String>, Vec<TokenStream>), TokenStream> {
    let mut skip_next = false;
    let mut depth = 0;
    let mut html_buffer: Vec<char> = vec![];
    let mut rust_buffer: Vec<char> = vec![];

    let mut html_parts: Vec<_> = vec![];
    let mut rust_evaluators: Vec<_> = vec![];
    let chars: Vec<_> = input.chars().collect();

    for (i, token) in chars.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        };

        let peek = chars.get(i + 1);
        let rust_mode = depth > 0;
        match token {
            '{' => {
                // Escaping bracket
                if !rust_mode && peek == Some(&'{') {
                    html_buffer.push('{');
                    skip_next = true;
                    continue;
                }
                depth += 1;
                if depth > 1 {
                    rust_buffer.push('{');
                }
            }
            '}' => {
                // Escaping bracket
                if !rust_mode && peek == Some(&'}') {
                    html_buffer.push('}');
                    skip_next = true;
                    continue;
                }
                depth -= 1;
                if depth > 0 {
                    rust_buffer.push('}');
                }
                if depth < 0 {
                    return Err(compile_error(
                        "Unexpected close bracket '}', need an open bracket first (or '}}' to escape)"
                    ));
                }
            }
            token => {
                if rust_mode {
                    rust_buffer.push(*token);
                } else {
                    html_buffer.push(*token);
                }
            }
        }

        let change_to_rust = !rust_mode && depth > 0;
        let change_to_html = rust_mode && depth == 0;

        // When exiting html, push html buffer
        if change_to_rust {
            let html_string: String = html_buffer.iter().collect();
            html_parts.push(html_string);
            html_buffer.clear();
        }

        // When exiting rust, verify and add to evaluators
        if change_to_html {
            let rust_string: String = rust_buffer.iter().collect();
            let rust_evaluator = match inner_rust_to_tokens(&rust_string) {
                Ok(rust_evaluator) => {
                    // Validate rust syntax is expr/ident/literal
                    let html_context: String = html_buffer.iter().collect();
                    let valid_rust =
                        validate_inner_rust(&rust_evaluator, &rust_string, &html_context);
                    match valid_rust {
                        Ok(_) => rust_evaluator,
                        Err(inner_err) => return Err(inner_err),
                    }
                }
                Err(err) => return Err(err),
            };
            rust_evaluators.push(rust_evaluator);
            rust_buffer.clear();
        }
    }

    if depth > 0 {
        return Err(compile_error("Missing close bracket '}'"));
    }

    let last_part = html_buffer.iter().collect();
    html_parts.push(last_part);

    Ok((html_parts, rust_evaluators))
}
