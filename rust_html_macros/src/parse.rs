use proc_macro2::TokenStream;

use html5ever::{
    interface::{QualName, QuirksMode},
    tokenizer::TokenizerOpts,
    tree_builder::TreeBuilderOpts,
    ParseOpts,
};
use scraper::HtmlTreeSink;
use tendril::TendrilSink;

macro_rules! qual_name {
    ($container:tt) => {
        QualName::new(None, ns!(html), local_name!($container))
    };
}

macro_rules! return_if_valid {
    ($container:tt, $html:ident) => {{
        let Err(error) = fragment_inside(qual_name!($container), $html) else {
            return Ok(());
        };
        error
    }};
}

pub fn compile_check_html(html: &str) -> Result<(), TokenStream> {
    let Err(errors) = validate_html(html) else {
        return Ok(());
    };
    // Convert to stream
    let error = crate::util::compile_error(&format!(
        "invalid HTML syntax ({} issues):\n{}",
        errors.len(),
        errors.join("\n")
    ));
    Err(error)
}

fn validate_html(html: &str) -> Result<(), Vec<String>> {
    // Check as body fragment or document root
    let fragment = scraper::Html::parse_fragment(html);
    if fragment.errors.is_empty() {
        return Ok(());
    };
    if scraper::Html::parse_document(html).errors.is_empty() {
        return Ok(());
    };

    // Custom fragments
    return_if_valid!("html", html);
    return_if_valid!("table", html);
    return_if_valid!("tr", html);

    Err(fragment
        .errors
        .into_iter()
        .map(|err| err.to_string())
        .collect())
}

pub fn fragment_inside(context: QualName, fragment: &str) -> Result<(), Vec<String>> {
    let parser = html5ever::driver::parse_fragment(
        HtmlTreeSink::new(scraper::Html::new_fragment()),
        ParseOpts {
            tokenizer: TokenizerOpts {
                exact_errors: true,
                ..Default::default()
            },
            tree_builder: TreeBuilderOpts {
                exact_errors: true,
                scripting_enabled: true,
                quirks_mode: QuirksMode::Quirks,
                ..Default::default()
            },
        },
        context,
        Vec::new(),
    );
    let html: scraper::Html = parser.one(fragment);
    if html.errors.is_empty() {
        Ok(())
    } else {
        Err(html.errors.into_iter().map(|v| v.to_string()).collect())
    }
}

/// Unit tests for HTML validation
#[cfg(test)]
mod test_html_validation {
    use crate::parse::validate_html;

    #[test]
    fn test_root() {
        valid(
            r#"
            <!DOCTYPE html>
            <html>
                <head></head>
                <body></body>
            </html>
        "#,
        );
        valid("<!DOCTYPE html><html></html>");
        invalid("<html></html>");
        valid("<body></body>");
    }

    #[test]
    fn test_div() {
        valid("<div>hello</div>");
        valid("<div class='hi'>hello</div>");
        invalid("<div/>");
    }

    #[test]
    fn test_button() {
        valid("<button>hello</button>");
        valid("<button></button>");
        invalid("<button/>");
    }

    #[test]
    fn test_br() {
        valid("<br/>");
        valid("<br><br>");
        invalid("<br></br>");
    }

    #[test]
    fn test_table() {
        valid(
            r#"
            <table>
                <tr><th>hi</th></tr>
                <tr><td>hi</td></tr>
            </table>
        "#,
        );
        valid(
            r#"
            <table>
                <thead>
                    <tr><th>hi</th></tr>
                    <tr><th>hi</th></tr>
                </thead>
                <tbody>
                    <tr><td>hi</td></tr>
                    <tr><td>hi</td></tr>
                </tbody>
            </table>
        "#,
        );
    }

    #[test]
    fn test_table_rows() {
        valid(
            r#"
            <tr>
                <td>hi world</td>
                <td>hi world</td>
            </tr>
        "#,
        );
    }

    #[test]
    fn test_table_fragments() {
        valid("<table></table>");
        valid("<thead></thead>");
        valid("<tbody></tbody>");
        valid("<tr></tr>");
        valid("<td>Table content</td>");
        valid("<th>Table content</th>");
    }

    #[test]
    fn test_script() {
        valid("<script></script>");
        valid("<script>alert('hi')</script>");
    }

    #[test]
    fn test_bad_attributes() {
        invalid("<div class='bad_class></div>");
        invalid("<div class=bad class></div>");
        invalid("<div class=></div>");
    }

    #[test]
    fn test_article() {
        valid("<article></article>");
    }

    #[test]
    fn test_fonts() {
        valid("<b>hello</b>");
        valid("<i>hello</i>");
        valid("<b><i>hello</i></b>");
    }

    #[test]
    fn test_iframe() {
        valid("<iframe></iframe>");
    }

    #[test]
    fn test_video() {
        valid("<video></video>");
    }

    #[test]
    fn test_input_label() {
        valid("<input />");
        valid("<label></label>");
        valid("<label><input/></label>");
    }

    #[test]
    fn test_comment() {
        valid("<!-- comment --><div></div>");
    }

    #[test]
    fn test_full_website() {
        valid(include_str!("./test.html"));
    }

    fn valid(html: &str) {
        let trimmed = crate::util::trim_whitespace_per_line(html);
        let result = validate_html(&trimmed);
        assert!(
            result.is_ok(),
            "{} is invalid: {}",
            &trimmed,
            result.unwrap_err().join(", ")
        );
    }

    fn invalid(html: &str) {
        let trimmed = crate::util::trim_whitespace_per_line(html);
        let result = validate_html(&trimmed);
        assert!(result.is_err(), "Expected not valid: {}", html);
    }
}
