use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::ToTokens;

/// Validates and extracts initial input to macro (input must be a single string literal)
pub fn validate_input(stream: TokenStream) -> Result<String, TokenStream> {
    let tokens: Vec<_> = stream.into_token_stream().into_iter().collect();
    if tokens.is_empty() {
        return Ok("".to_string());
    }
    if tokens.len() > 1 {
        let error = format!("expected a single input, but found {}", tokens.len());
        return Err(compile_error(&error));
    }
    match litrs::Literal::try_from(tokens.first().unwrap()) {
        Err(e) => Err(e.to_compile_error().into()),
        Ok(litrs::Literal::String(value)) => Ok(value.value().to_string()),
        Ok(other_type) => {
            let error = format!(
                "expected string literal, but found literal '{}'",
                other_type
            );
            Err(compile_error(&error))
        }
    }
}

/// Verifies that rust code is an identifier, expression or literal
pub fn validate_inner_rust(
    code: &TokenStream,
    code_string: &str,
    html_context: &str,
) -> Result<(), TokenStream> {
    let Err(ident_err) = syn::parse2::<syn::Ident>(code.clone()) else {
        return Ok(());
    };
    if syn::parse2::<syn::Lit>(code.clone()).is_ok() {
        return Ok(());
    }
    if syn::parse2::<syn::Expr>(code.clone()).is_ok() {
        return Ok(());
    }
    Err(wrap_rust_compile_error(
        "template input is not a valid identifer/expression/literal: ",
        ident_err,
        code_string,
        html_context,
    ))
}

/// Converts string to proc macro literal
pub fn string_to_literal(string: &str) -> TokenStream {
    [TokenTree::Literal(Literal::string(string))]
        .into_iter()
        .collect()
}

/// Converts rust code to TokenStream
/// If the code is empty, default to empty "" string literal
pub fn inner_rust_to_tokens(rust_code: &str) -> Result<TokenStream, TokenStream> {
    if rust_code.trim() == "" {
        return Ok("\"\"".parse().expect("failed due to empty {{}} template"));
    }
    match rust_code.parse() {
        Ok(token_stream) => Ok(token_stream),
        Err(_err) => {
            let error = format!(
                r#"failed to parse rust tokens '{}' inside brackets '{{}}'"#,
                rust_code
            );
            Err(compile_error(&error))
        }
    }
}

/// Utility for generating a nice inner rust compile error
pub fn wrap_rust_compile_error(
    prefix: &str,
    mut rust_error: syn::Error,
    rust_code: &str,
    html_code: &str,
) -> TokenStream {
    let mut html_short = false;
    let html_info_max = 20;
    let html_info = if html_code.is_empty() {
        ""
    } else if html_code.len() <= html_info_max {
        &html_code[..html_code.len() - 1]
    } else {
        html_short = true;
        &html_code[(html_code.len() - html_info_max)..html_code.len() - 1]
    };

    let mut rust_short = false;
    let rust_info_max = 15;
    let rust_info = if rust_code.is_empty() {
        ""
    } else if rust_code.len() <= rust_info_max {
        rust_code
    } else {
        rust_short = true;
        &rust_code[..(rust_code.len() - rust_info_max)]
    };

    let message = format!(
        "'{}{}{}{}}}' <-  {}",
        if html_short { "... " } else { "" },
        html_info,
        rust_info,
        if rust_short { " ..." } else { "" },
        prefix
    );
    let context_error = syn::Error::new(rust_error.span(), message);
    rust_error.combine(context_error);
    rust_error.into_compile_error()
}

/// Utility for returning a compile error stream
pub fn compile_error(error: &str) -> TokenStream {
    [
        TokenTree::Ident(proc_macro2::Ident::new("compile_error", Span::mixed_site())),
        TokenTree::Punct(Punct::new('!', Spacing::Alone)),
        TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            [TokenTree::Literal(Literal::string(error))]
                .into_iter()
                .collect(),
        )),
    ]
    .into_iter()
    .collect()
}

pub fn validate_html(html: &str) -> Result<(), TokenStream> {
    let fragment = scraper::Html::parse_fragment(html);
    if fragment.errors.is_empty() {
        return Ok(());
    };
    if scraper::Html::parse_document(html).errors.is_empty() {
        return Ok(());
    };

    // Error hint for root html
    let mut all_errors = fragment.errors.join(",\n");
    if fragment.errors.last() == Some(&"</html> with no <body> in scope".into())
        && html.contains("<html>")
    {
        all_errors = format!(
            "{},\n{}",
            "<!DOCTYPE html> is required for root html", all_errors
        );
    }

    let error = compile_error(&format!(
        "invalid HTML syntax ({} issues):\n{}",
        fragment.errors.len(),
        all_errors
    ));
    Err(error)
}
