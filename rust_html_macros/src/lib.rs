use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod util;
use util::*;

const REPLACE_TARGET_PREFIX: &str = "__RHTML_REPLACE_TARGET__";

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
/// Note that these imports use the rust_html_macros crate.
/// In your project you need to use `rust_html` instead,
/// which depends on this crate.
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
/// let class = "my_class";
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
    let (html_string, rust_evaluators) = match parse_rhtml(&input_string) {
        Ok(result) => result,
        Err(err) => return err,
    };

    // Compile time HTML syntax check
    let html_for_validate = create_template_for_compile_check(&html_string, rust_evaluators.len());
    if let Err(error) = validate_html(&html_for_validate) {
        return error;
    }

    // Build output TokenStream
    let html_literal = string_to_literal(&html_string);
    let param_vec_ident = format_ident!("parameters");
    let param_val_ident: Vec<_> = rust_evaluators
        .iter()
        .enumerate()
        .map(|(i, _)| format_ident!("param_{}", i))
        .collect();
    quote! {
        {
            use rust_html::{Render, Template};
            let mut #param_vec_ident: Vec<Box<dyn rust_html::Render>> = vec![];
            #(
                let #param_val_ident = {#rust_evaluators.clone()}.to_owned();
                #param_vec_ident.push(Box::new(#param_val_ident));
            )*
            rust_html::Template::build_internal(
                #html_literal,
                #param_vec_ident,
            )
        }
    }
}

/// Replaces all injections with an empty string and
/// for validating the resulting HTML template string
fn create_template_for_compile_check(html: &str, n_evaluators: usize) -> String {
    let mut template: String = html.to_owned();
    for i in 0..n_evaluators {
        let target = format!("{{{}{}}}", REPLACE_TARGET_PREFIX, i);
        template = template.replace(&target, "");
    }
    template
}

/// Parses rhtml content. On success returns HTML string template and list of
/// rust token streams to inject into the string.
fn parse_rhtml(input: &str) -> Result<(String, Vec<TokenStream>), TokenStream> {
    let mut skip_next = false;
    let mut depth = 0;
    let mut html_buffer: Vec<char> = vec![];
    let mut rust_buffer: Vec<char> = vec![];
    let mut rust_evaluators: Vec<_> = vec![];
    let chars: Vec<_> = input.chars().collect();

    for (i, token) in chars.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        };

        let peek = chars.get(i + 1);
        let mut exit_rust = false;
        match token {
            '{' => {
                depth += 1;

                // Escaping bracket
                if depth == 1 && peek == Some(&'{') {
                    html_buffer.push('{');
                    skip_next = true;
                    depth = 0;
                    continue;
                }

                // Entered rust code (insert replace target)
                if depth == 1 && rust_buffer.is_empty() {
                    html_buffer.push('{');
                    html_buffer.extend(REPLACE_TARGET_PREFIX.to_string().chars());
                    html_buffer.extend(rust_evaluators.len().to_string().chars());
                    html_buffer.push('}');
                } else {
                    // Bracket inside rust code
                    rust_buffer.push('{');
                }
            }
            '}' => {
                depth -= 1;

                // Escaping bracket
                if depth == -1 && peek == Some(&'}') {
                    html_buffer.push('}');
                    skip_next = true;
                    depth = 0;
                    continue;
                }
                match depth {
                    0 => exit_rust = true,
                    depth => {
                        if depth > 0 {
                            rust_buffer.push('}');
                        } else {
                            html_buffer.push('{')
                        }
                    }
                }
            }
            token => {
                if depth == -1 && rust_buffer.is_empty() {
                    return Err(compile_error("missing open bracket '{' before closing bracket '}' (or to escape, use '}}')"));
                }
                if depth > 0 {
                    rust_buffer.push(*token);
                } else {
                    html_buffer.push(*token);
                }
            }
        }

        // When exiting rust, verify and add to evaluators
        if exit_rust {
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

    if !rust_buffer.is_empty() || depth > 0 {
        return Err(compile_error("Missing close bracket '}'"));
    }

    let html_string: String = html_buffer.iter().collect();
    Ok((html_string, rust_evaluators))
}
