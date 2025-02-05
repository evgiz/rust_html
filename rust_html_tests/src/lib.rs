// Unit tests for RHTML crate
#[cfg(test)]
mod test {
    use rust_html::*;

    #[test]
    pub fn test_empty() {
        test_eq(rhtml! {}, "");
    }

    #[test]
    pub fn test_empty_inner() {
        test_eq(rhtml! {"{}"}, "");
    }

    #[test]
    pub fn test_template_only() {
        test_eq(
            rhtml! { "<div>hello, world</div>" },
            "<div>hello, world</div>",
        );
    }

    #[test]
    pub fn test_constant_only() {
        test_eq(rhtml! {"{10}"}, "10");
    }

    #[test]
    pub fn test_multiple_constants_only() {
        test_eq(rhtml! {"{10}{20}"}, "1020");
    }

    #[test]
    pub fn test_multiple_constants_with_space() {
        test_eq(rhtml! {"{10} {20}"}, "10 20");
    }

    #[test]
    pub fn test_int_variable() {
        let value = 10;
        test_eq(rhtml! {"{value}"}, "10");
    }

    #[test]
    pub fn test_int_reference_variable() {
        let value1 = 10;
        test_eq(rhtml! {"{value1}"}, "10");
        let value2 = &20;
        test_eq(rhtml! {"{value2}"}, "20");
    }

    #[test]
    pub fn test_str_variable() {
        let value = "hello";
        test_eq(rhtml! {"{value}"}, "hello");
    }

    #[test]
    pub fn test_string_variable() {
        let value = "hello".to_string();
        test_eq(rhtml! {"{value}"}, "hello");
    }

    #[test]
    pub fn test_float_variable() {
        let value = 5.3;
        test_eq(rhtml! {"{value}"}, "5.3");
    }

    #[test]
    pub fn test_closure_variable() {
        let closure = |subject: &str| {
            let hello = "hello";
            format!("{}, {}", hello, subject)
        };
        test_eq(rhtml! {r#"{closure("world")}"#}, "hello, world");
    }

    #[test]
    pub fn test_constant_in_template() {
        test_eq(rhtml! {"<div>{10}</div>"}, "<div>10</div>");
    }

    #[test]
    pub fn test_block_in_rust_evaluator() {
        test_eq(
            rhtml! {r#"{ { "string_in_rust_block" } }"#},
            "string_in_rust_block",
        );
    }

    #[test]
    pub fn test_html_class() {
        let my_class = "button";
        test_eq(
            rhtml! {r#"<div class="{my_class}"></div>"#},
            "<div class=\"button\"></div>",
        );
    }

    #[test]
    pub fn test_profile_card() {
        let name_cls = "green";
        let user_name = "evgiz";
        test_eq(
            rhtml! {
            r#"
                <div class="container">
                    <div class="name {name_cls}">
                        {user_name}
                    </div>
                    <div class="title">
                        Rust Programmer
                    </div>
                </div>
            "#},
            r#"
                <div class="container">
                    <div class="name green">
                        evgiz
                    </div>
                    <div class="title">
                        Rust Programmer
                    </div>
                </div>
            "#,
        );
    }

    #[test]
    pub fn test_composed() {
        let name = "evgiz";
        let card = rhtml! {r#"<div class="card">{name}</div>"#};
        test_eq(
            rhtml! {r#"
            <div class="page">
                {card}
            </div>
        "#},
            r#"
            <div class="page">
                <div class="card">evgiz</div>
            </div>
        "#,
        );
    }

    #[test]
    pub fn test_composed_closure() {
        let name = "evgiz";
        let card = |content: &str| {
            rhtml! {r#"<div class="card">{content}</div>"#}
        };
        test_eq(
            rhtml! {r#"
            <div class="page">
                {card(name)}
            </div>
        "#},
            r#"
            <div class="page">
                <div class="card">evgiz</div>
            </div>
        "#,
        );
    }

    #[test]
    pub fn test_escaping_brackets_both() {
        test_eq(rhtml! {"{{}}"}, "{}");
    }

    #[test]
    pub fn test_escaping_brackets_left() {
        test_eq(rhtml! {"{{"}, "{");
    }

    #[test]
    pub fn test_escaping_brackets_right() {
        test_eq(rhtml! {"}}"}, "}");
    }

    #[test]
    pub fn test_multiple_escaping_brackets() {
        test_eq(rhtml! {"{{}}hello{{}}"}, "{}hello{}");
        test_eq(rhtml! {"{{}}{{}}"}, "{}{}");
    }

    #[test]
    pub fn test_manual_injection_target() {
        test_eq(rhtml! {"{0}"}, "0");
    }

    #[test]
    pub fn test_manual_injection_target_escape() {
        test_eq(rhtml! {"{{0}}{1}"}, "{0}1");
    }

    #[test]
    pub fn test_conditional() {
        test_eq(rhtml! {"{ if true {1} else {2} }"}, "1");
    }

    #[test]
    pub fn test_conditional_str() {
        test_eq(rhtml! {r#"{ if true { "hello" } else {"world"}}"#}, "hello");
    }

    #[test]
    pub fn test_conditional_template_or_str() {
        let template = || rhtml! { "hello" };
        let string = "world".to_string();
        test_eq(
            rhtml! {r#"{ if true { template() } else { string.into() } }"#},
            "hello",
        );
    }

    #[test]
    pub fn test_escape_html() {
        let sketchy_user_input = "<script>alert('hi')</script>";
        test_eq(
            rhtml! {"<div>{sketchy_user_input}</div>"},
            "<div>&lt;script&gt;alert(&#x27;hi&#x27;)&lt;&#x2F;script&gt;</div>",
        );
    }

    #[test]
    pub fn test_escape_html_class() {
        let sketchy_user_input = "class\"sketchy";
        test_eq(
            rhtml! {r#"<div class="{sketchy_user_input}"></div>"#},
            r#"<div class="class&quot;sketchy"></div>"#,
        );
    }

    #[test]
    pub fn test_unescaped() {
        let sketchy_user_input = "<script>alert('hi')</script>";
        let unescaped = Unescaped(sketchy_user_input.to_string());
        test_eq(
            rhtml! {r#"<div>{unescaped}</div>"#},
            r#"<div><script>alert('hi')</script></div>"#,
        );
    }

    #[test]
    pub fn test_render_trait() {
        #[derive(Clone)]
        struct Component {
            text: String,
        }
        impl Render for Component {
            fn render(&self) -> Template {
                rhtml! {r#"<span>{self.text}</span>"#}
            }
        }
        let component = Component {
            text: "hello, world".into(),
        };
        test_eq(
            rhtml! {"<div>{component}</div>"},
            "<div><span>hello, world</span></div>",
        );
    }

    #[test]
    pub fn test_json_text() {
        let calculate = || 10.0;
        test_eq(
            rhtml! {r#"
        {{
            "name": "evgiz",
            "code_speed": {calculate()}
        }}
        "#},
            r#"
        {
            "name": "evgiz",
            "code_speed": 10
        }
        "#,
        );
    }

    #[test]
    pub fn test_impl_render_input() {
        let page = rhtml! {r#"{component_dummy("hello, world")}"#};
        test_eq(page, "<div>hello, world</div>");
    }

    #[test]
    pub fn test_nested_impl_render_input() {
        let text = "hello, world";
        let inner = rhtml! {r#"<span>{text}</span>"#};
        let page = rhtml! {r#"{component_dummy(inner)}"#};
        test_eq(page, "<div><span>hello, world</span></div>");
    }

    fn component_dummy(children: impl Render + Clone) -> Template {
        rhtml! {"<div>{children}</div>"}
    }

    struct ComponentTest {
        content: Template,
    }

    impl Render for ComponentTest {
        fn render(&self) -> Template {
            rhtml! {"<div>{self.content}</div>"}
        }
    }

    #[test]
    pub fn test_struct_component() {
        let component = ComponentTest {
            content: rhtml! {"hello, world"},
        };
        let page = rhtml! {"<div>{component}</div>"};
        test_eq(page, "<div><div>hello, world</div></div>");
    }

    #[test]
    pub fn test_html_root() {
        // Just a compile ok check
        rhtml! {r#"
            <!DOCTYPE html>
            <html>
                <head></head>
                <body></body>
            </html>
        "#};
    }

    #[test]
    pub fn test_html_table() {
        test_eq(rhtml! {"<tr></tr>"}, "<tr></tr>");
        test_eq(rhtml! {"<td>{10}</td>"}, "<td>10</td>");
        test_eq(rhtml! {"<th>{10}</th>"}, "<th>10</th>");
    }

    fn test_eq(template: Template, expected: &str) {
        let template_string: String = template.into();
        assert!(
            template_string == expected,
            "Macro test failed, expected:\n {}\nbut found:\n {}\n",
            expected,
            template_string
        )
    }
}
