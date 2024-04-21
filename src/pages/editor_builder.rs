use html_tag::HtmlTag;

/// Edit pages are rather regular and share many of the same components so rather than
/// hand-code a whole bunch of html, templating, and javascript we'll generate all that.
pub struct EditorBuilder {
    head: HtmlTag,
    prolog: Vec<HtmlTag>, // first half of body
    form: HtmlTag,        // second half of body
}

impl EditorBuilder {
    /// Note that the builder will escape URLs.
    pub fn new(post_url: &str) -> EditorBuilder {
        fn make_head() -> HtmlTag {
            let mut head = HtmlTag::new("head");
            head.add_child(HtmlTag::new("meta").with_attribute("charset", "utf-8"));
            head.add_child(
                HtmlTag::new("meta")
                    .with_attribute("name", "viewport")
                    .with_attribute("content", "width=device-width, initial-scale=1"),
            );
            head.add_child(HtmlTag::new("title").with_body("web lifter"));
            head.add_child(
                HtmlTag::new("link")
                    .with_attribute(
                        "href",
                        "https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css",
                    )
                    .with_attribute("rel", "stylesheet")
                    .with_attribute(
                        "integrity",
                        "sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH",
                    )
                    .with_attribute("crossorigin", "anonymous"),
            );
            head.add_child(
                HtmlTag::new("link")
                    .with_attribute("href", "/styles/style.css?version=2")
                    .with_attribute("rel", "stylesheet"),
            );
            head
        }

        fn make_prolog() -> Vec<HtmlTag> {
            let script = HtmlTag::new("script")
                .with_attribute(
                    "src",
                    "https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js",
                )
                .with_attribute(
                    "integrity",
                    "sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz",
                )
                .with_attribute("crossorigin", "anonymous");
            vec![script]
        }

        fn make_form(post_url: &str) -> HtmlTag {
            let post_url = url_escape::encode_path(post_url);
            HtmlTag::new("form")
                .with_attribute("method", "post")
                .with_attribute("action", &post_url)
                .with_attribute("role", "form")
                .with_attribute("class", "mt-4 ms-2 me-2")
        }

        EditorBuilder {
            head: make_head(),
            prolog: make_prolog(),
            form: make_form(post_url),
        }
    }

    /// Call this or with_edit_dropdown.
    pub fn with_title(mut self, title: &str) -> EditorBuilder {
        let title = HtmlTag::new("h2")
            .with_class("text-center")
            .with_body(title);
        self.prolog.push(title);
        self
    }

    pub fn with_float_input(
        mut self,
        name: &str,
        value: &str,
        min: &str,
        step: &str,
        help: &str,
    ) -> EditorBuilder {
        let key = name.to_lowercase();
        let key = key.replace(" ", "-");

        let label_id = format!("{key}-label");
        let input_id = format!("{key}-input");
        let help_id = format!("{key}-help");

        let mut div = HtmlTag::new("div").with_class("input-group");
        div.add_child(
            HtmlTag::new("span")
                .with_id(&label_id)
                .with_class("input-group-text")
                .with_body(name),
        );
        let mut input = HtmlTag::new("input")
            .with_id(&input_id)
            .with_class("form-control")
            .with_attribute("type", "number")
            .with_attribute("name", &key)
            .with_attribute("min", min)
            .with_attribute("step", step)
            .with_attribute("aria-describedby", &format!("{label_id} {help_id}"))
            .with_attribute("value", value);
        if !has_children(&self.form) {
            input.add_attribute("autofocus", "");
        }
        div.add_child(input);
        self.form.add_child(div);

        let help = HtmlTag::new("div")
            .with_id(&help_id)
            .with_class("form-text fst-italic fs-6")
            .with_body(help);
        self.form.add_child(help);
        self
    }

    /// Adds Cancel and Save buttons. Called immediately before finalize.
    pub fn with_std_buttons(mut self, cancel_url: &str) -> EditorBuilder {
        let cancel_url = url_escape::encode_path(cancel_url);

        let mut div = HtmlTag::new("div").with_class("form-group mt-4");
        let mut div2 = HtmlTag::new("div").with_class("row justify-content-evenly");

        let mut div3 = HtmlTag::new("div").with_class("col-4 align-self-center");
        div3.add_child(
            HtmlTag::new("button")
                .with_class("btn btn-secondary")
                .with_body("Cancel")
                .with_attribute("type", "submit")
                .with_attribute("formmethod", "get")
                .with_attribute("formaction", &cancel_url)
                .with_attribute("formnovalidate", ""),
        );
        div2.add_child(div3);

        let mut div3 = HtmlTag::new("div").with_class("col-4 align-self-center");
        div3.add_child(
            HtmlTag::new("button")
                .with_class("btn btn-primary")
                .with_body("Save")
                .with_attribute("type", "submit"),
        );
        div2.add_child(div3);
        div.add_child(div2);
        self.form.add_child(div);
        self
    }

    pub fn finalize(self) -> String {
        let mut text = String::with_capacity(6 * 1024);
        text += "<!DOCTYPE html>\n";
        text += "<html lang='en'>\n\n";

        text += &self.head.construct();
        text += "\n";

        text += "<body style='--bs-body-font-size: 1.25rem'>\n";
        for t in self.prolog {
            text += &t.construct();
        }
        text += "\n";

        text += &self.form.construct();
        text += "</body>\n";
        text += "</html>\n";
        text
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn any_weight() {
//         let html = EditorBuilder::new("/set-any-weight/Heavy/Bench")
//             .with_title("Edit Weight")
//             .with_float_input(
//                 "Weight",
//                 "135.0",
//                 "0",
//                 "0.01",
//                 "Arbitrary weight (i.e. there isn't a weight set).",
//             )
//             .with_std_buttons("/exercise/Heavy/Bench")
//             .finalize();
//         println!("{html}");
//         assert_eq!("bar", "");
//     }
// }

fn has_children(tag: &HtmlTag) -> bool {
    if let Some(children) = &tag.children {
        !children.is_empty()
    } else {
        false
    }
}
