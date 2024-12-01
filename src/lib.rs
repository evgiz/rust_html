use std::fmt::Display;

pub use rust_html_macros::rhtml;

/// Struct representing a rust_html template.
/// Enables easy reusability and injection safety.
///
/// Create one using the `rust_html::rhtml!` macro:
///
/// ```rust
/// use rust_html::{rhtml, Template};
/// let my_template: Template = rhtml! { "<div>Hello!<div> "};
/// ```
///
/// You can convert a template to a HTML string:
///
/// ```rust
/// use rust_html::{rhtml, Template};
/// let template = rhtml! { "<div>hello, world</div>"};
/// let html: String = template.into();
/// ```
///
/// You should typically only do this just before
/// returning the HTML in your endpoint.
///
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Template {
    content: TemplateContent,
}

/// Represents a group of rust_html templates
///
/// Use this wrapper if you need to insert a
/// `Vec<Template>` into another template.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TemplateGroup(pub Vec<Template>);

/// Wrapper to insert unescaped content into
/// a rust_html template. Never use Unescaped
/// on untrusted user input!
///
/// ```rust
/// use rust_html::{rhtml, Unescaped};
/// let sketchy_user_input = "<script>".to_string();
///
/// let safe_template = rhtml! { "{sketchy_user_input}" };
/// assert_eq!(String::from(safe_template), "&lt;script&gt;");
///
/// let unescaped = Unescaped(sketchy_user_input.clone());
/// let unsafe_template = rhtml! { "{unescaped}" };
/// assert_eq!(String::from(unsafe_template), "<script>");
/// ```
///
#[derive(Debug, Clone)]
pub struct Unescaped(pub String);

/// Render trait for rust_html templates
///
/// Implement this trait on a struct to create
/// reusable struct components that you
/// can reuse inside other templates.
pub trait Render {
    fn render(&self) -> Template;
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum TemplateContent {
    RawString(String),
    WithParameters {
        template: String,
        parameters: Vec<Template>,
    },
}

// Internal replace target. Don't use this in your HTML :)
const REPLACE_TARGET_PREFIX: &str = "__RHTML_REPLACE_TARGET__";

impl Template {
    /// Internal macro creation of a rust_html template.
    ///
    /// DO NOT USE THIS.
    /// USE THE `rhtml!` MACRO.
    ///
    /// This implementation is very low level, and requires use of the
    /// internal rust_html templating syntax.
    pub fn build_internal(template: impl Display, parameters: Vec<Box<dyn Render>>) -> Self {
        Template {
            content: TemplateContent::WithParameters {
                template: template.to_string(),
                parameters: parameters.into_iter().map(|r| r.render()).collect(),
            },
        }
    }
    /// Internal function. Converts a template to String
    fn build(&self) -> String {
        match &self.content {
            TemplateContent::RawString(value) => value.to_owned(),
            TemplateContent::WithParameters {
                template,
                parameters,
            } => {
                let mut output = template.clone();
                for (i, param) in parameters.iter().enumerate() {
                    let target = format!("{{{}{}}}", REPLACE_TARGET_PREFIX, i);
                    let replace: String = param.build();
                    output = output.replace(&target, replace.trim());
                }
                output
            }
        }
    }
}

impl Render for Template {
    fn render(&self) -> Template {
        self.clone()
    }
}

impl Render for Unescaped {
    fn render(&self) -> Template {
        Template {
            content: TemplateContent::RawString(self.0.to_owned()),
        }
    }
}

impl Render for TemplateGroup {
    fn render(&self) -> Template {
        let mut injects = vec![];
        let mut parameters = vec![];
        for (i, template) in self.0.iter().enumerate() {
            injects.push(format!("{{{}{}}}", REPLACE_TARGET_PREFIX, i));
            parameters.push(template.clone());
        }
        let template = injects.join("\n");
        Template {
            content: TemplateContent::WithParameters {
                template,
                parameters,
            },
        }
    }
}

impl<I> From<I> for TemplateGroup
where
    I: IntoIterator<Item = Template>,
{
    fn from(value: I) -> Self {
        TemplateGroup(value.into_iter().collect())
    }
}

impl std::iter::FromIterator<Template> for TemplateGroup {
    fn from_iter<T: IntoIterator<Item = Template>>(iter: T) -> Self {
        TemplateGroup(iter.into_iter().collect())
    }
}

impl<T> Render for T
where
    T: std::fmt::Display,
{
    fn render(&self) -> Template {
        let string = self.to_string();
        let escaped_value = html_escape::encode_safe(&string);
        Template {
            content: TemplateContent::RawString(escaped_value.into()),
        }
    }
}

impl From<Template> for String {
    fn from(value: Template) -> Self {
        value.build()
    }
}
