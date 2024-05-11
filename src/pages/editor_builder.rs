use std::collections::HashMap;

use html_tag::HtmlTag;

pub struct EditorBuilder {
    head: HtmlTag,
    prolog: Vec<HtmlTag>, // first half of body
    form: HtmlTag,        // second half of body
    custom: String,       // arbitrary HTML added after the form
}

pub trait Widget {
    fn build(&self, builder: &mut EditorBuilder);
}

/// Edit pages are rather regular and share many of the same components so rather than
/// hand-code a whole bunch of html, templating, and javascript we'll generate all that.
/// Note that the builder will escape URLs.
pub fn build_editor(post_url: &str, widgets: Vec<Box<dyn Widget>>) -> String {
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

    fn finalize(builder: EditorBuilder) -> String {
        let mut text = String::with_capacity(6 * 1024);
        text += "<!DOCTYPE html>\n";
        text += "<html lang='en'>\n\n";

        text += &builder.head.construct();
        text += "\n";

        text += "<body style='--bs-body-font-size: 1.25rem'>\n";
        for t in builder.prolog {
            text += &t.construct();
        }
        text += "\n";
        text += "\n";

        text += &builder.form.construct();
        if !builder.custom.is_empty() {
            text += &builder.custom;
        }
        text += "</body>\n";
        text += "</html>\n";
        text
    }

    let mut builder = EditorBuilder {
        head: make_head(),
        prolog: make_prolog(),
        form: make_form(post_url),
        custom: String::new(),
    };
    for widget in widgets {
        widget.build(&mut builder);
    }
    finalize(builder)
}

// =======================================================================================
/// List of checkbox buttons.
pub struct Checkbox {
    name: String,
    items: Vec<(String, String, bool)>,
    help: String,
    javascript: String,
}

impl Checkbox {
    /// Construct checkboxes with a vector of (label, name, checked) entries.
    pub fn new(name: &str, items: Vec<(String, String, bool)>, help: &str) -> Checkbox {
        let items = items.into_iter().map(|(l, v, c)| (l, v, c)).collect();
        Checkbox {
            name: name.to_owned(),
            items,
            help: help.to_owned(),
            javascript: include_str!("../../files/checkboxes.js").to_owned(),
        }
    }

    // pub fn with_custom_js(self, javascript: &str) -> Checkbox {
    //     Checkbox {
    //         javascript: javascript.to_owned(),
    //         ..self
    //     }
    // }

    // pub fn without_js(self) -> Checkbox {
    //     Checkbox {
    //         javascript: "".to_owned(),
    //         ..self
    //     }
    // }
}

impl Widget for Checkbox {
    fn build(&self, builder: &mut EditorBuilder) {
        // checkboxes
        for item in self.items.iter() {
            let mut entry = HtmlTag::new("div").with_class("form-check");
            let mut input = HtmlTag::new("input")
                .with_id(&format!("{}-btn", item.1))
                .with_class("form-check-input")
                .with_attribute("type", "checkbox")
                // .with_attribute("name", &self.name)
                .with_attribute("onclick", "on_click()")
                .with_attribute("value", &item.1);
            if item.2 {
                input.add_attribute("checked", "");
            }
            entry.add_child(input);

            let label = HtmlTag::new("label")
                .with_class("form-check-label")
                .with_attribute("for", &format!("{}-btn", item.1))
                .with_body(&item.0);
            entry.add_child(label);
            builder.form.add_child(entry);
        }

        // help
        let div = HtmlTag::new("div")
            .with_id(&format!("{}-help", self.name))
            .with_class("form-text fst-italic fs-6 mb-3")
            .with_body(&self.help);
        builder.form.add_child(div);

        // hidden button (used with post to send the selected item name)
        let button = HtmlTag::new("input")
            .with_id("check-values")
            .with_attribute("type", "text")
            .with_attribute("name", &self.name)
            .with_attribute("value", "")
            .with_attribute("hidden", "");
        builder.form.add_child(button);

        // javascript
        if !self.javascript.is_empty() {
            let js = HtmlTag::new("script")
                .with_attribute("type", "text/javascript")
                .with_body(&self.javascript);
            builder.prolog.push(js);
        }
    }
}

// =======================================================================================
/// Button that shows a dropdown list when clicked.
pub struct Dropdown {
    label: String,
    active: String,
    items: Vec<(String, String)>,
    javascript: String,
    help: String,
}

impl Dropdown {
    /// Construct dropdown list with a slice of (body, value) entries.
    pub fn new(label: &str, items: &[(&str, &str)], javascript: &str) -> Dropdown {
        let items = items
            .iter()
            .map(|(b, v)| (v.to_string(), b.to_string()))
            .collect();
        Dropdown {
            label: label.to_owned(),
            active: "".to_owned(),
            items,
            javascript: javascript.to_owned(),
            help: "".to_owned(),
        }
    }

    /// Name of the item to mark as selected on page load.
    pub fn with_active(self, active: &str) -> Dropdown {
        Dropdown {
            active: active.to_owned(),
            ..self
        }
    }

    pub fn with_help(self, help: &str) -> Dropdown {
        Dropdown {
            help: help.to_owned(),
            ..self
        }
    }
}

impl Widget for Dropdown {
    fn build(&self, builder: &mut EditorBuilder) {
        let key = self.label.to_lowercase();
        let key = key.replace(" ", "-");

        let select_id = format!("{key}-select");
        let help_id = format!("{key}-help");

        let mut div = HtmlTag::new("div").with_class("input-group");
        let span = HtmlTag::new("span")
            .with_class("input-group-text")
            .with_body(&self.label);
        div.add_child(span);

        let mut select = HtmlTag::new("select")
            .with_id(&select_id)
            .with_class("form-select")
            .with_attribute("aria-label", &key)
            .with_attribute("aria-describedby", &help_id)
            .with_attribute("name", &key);
        for item in self.items.iter() {
            let mut option = HtmlTag::new("option")
                .with_attribute("value", &item.0)
                .with_body(&item.1);
            if item.1 == self.active {
                option.add_attribute("selected", "");
            }
            select.add_child(option);
        }
        div.add_child(select);
        builder.form.add_child(div);

        if !self.help.is_empty() {
            let div = HtmlTag::new("div")
                .with_id(&help_id)
                .with_class("form-text fst-italic fs-6")
                .with_body(&self.help);
            builder.form.add_child(div);
        }

        // javascript
        if !self.javascript.is_empty() {
            let js = HtmlTag::new("script")
                .with_attribute("type", "text/javascript")
                .with_body(&self.javascript);
            builder.prolog.push(js);
        }
    }
}

// =======================================================================================
/// Floating point text field.
pub struct FloatInput {
    label: String,
    value: String,
    min: String,
    step: String,
    help: String,
    required: bool,
}

impl FloatInput {
    pub fn new(label: &str, value: Option<f32>, help: &str) -> FloatInput {
        FloatInput {
            label: label.to_owned(),
            value: value.map_or("".to_owned(), |v| FloatInput::format_float(v)),
            min: FloatInput::format_float(0.0),
            step: FloatInput::format_float(0.01),
            help: help.to_owned(),
            required: false,
        }
    }

    /// Defaults to 0.0
    pub fn with_min(self, min: f32) -> FloatInput {
        FloatInput {
            min: FloatInput::format_float(min),
            ..self
        }
    }

    // Defaults to 0.01
    pub fn with_step(self, step: f32) -> FloatInput {
        FloatInput {
            step: FloatInput::format_float(step),
            ..self
        }
    }

    /// User has to enter something into the field in order to pass client side validation.
    pub fn with_required(self) -> FloatInput {
        FloatInput {
            required: true,
            ..self
        }
    }

    fn format_float(value: f32) -> String {
        let mut s = format!("{value:.3}");
        while s.ends_with("0") {
            s.remove(s.len() - 1);
        }
        if s.ends_with(".") {
            s.remove(s.len() - 1);
        }
        s
    }
}

impl Widget for FloatInput {
    fn build(&self, builder: &mut EditorBuilder) {
        let key = self.label.to_lowercase();
        let name = key.replace(" ", "_");
        let key = key.replace(" ", "-");

        let label_id = format!("{key}-label");
        let input_id = format!("{key}-input");
        let help_id = format!("{key}-help");

        let mut div = HtmlTag::new("div").with_class("input-group");
        div.add_child(
            HtmlTag::new("span")
                .with_id(&label_id)
                .with_class("input-group-text")
                .with_body(&self.label),
        );
        let mut input = HtmlTag::new("input")
            .with_id(&input_id)
            .with_class("form-control")
            .with_attribute("type", "number")
            .with_attribute("name", &name)
            .with_attribute("min", &self.min)
            .with_attribute("step", &self.step)
            .with_attribute("aria-describedby", &format!("{label_id} {help_id}"))
            .with_attribute("value", &self.value);
        if !has_children(&builder.form) {
            input.add_attribute("autofocus", "");
        }
        if self.required {
            input.add_attribute("required", "");
        }
        div.add_child(input);
        builder.form.add_child(div);

        let help = HtmlTag::new("div")
            .with_id(&help_id)
            .with_class("form-text fst-italic fs-6 mb-4")
            .with_body(&self.help);
        builder.form.add_child(help);
    }
}

// =======================================================================================
/// Hidden input field used with javascript.
pub struct HiddenInput {
    name: String,
}

impl HiddenInput {
    pub fn new(name: &str) -> HiddenInput {
        HiddenInput {
            name: name.to_owned(),
        }
    }
}

impl Widget for HiddenInput {
    fn build(&self, builder: &mut EditorBuilder) {
        let key = self.name.to_lowercase();
        let key = key.replace(" ", "-");

        let id = format!("{key}-btn");
        let input = HtmlTag::new("input")
            .with_id(&id)
            .with_attribute("type", "text")
            .with_attribute("name", &self.name)
            .with_attribute("value", "")
            .with_attribute("hidden", "");
        builder.form.add_child(input);
    }
}

// =======================================================================================
/// Arbitrary injected HTML, used for things like modals.
pub struct Html {
    content: String,
}

impl Html {
    pub fn new(content: &str) -> Html {
        Html {
            content: content.to_owned(),
        }
    }
}

impl Widget for Html {
    fn build(&self, builder: &mut EditorBuilder) {
        builder.custom += &self.content;
    }
}

// =======================================================================================
#[derive(Eq, PartialEq)]
enum Help {
    None,
    PerItem,
    Hardcoded(String),
}

/// Selectable item list. If active matches an item name then that item will start
/// out selected.
pub struct List {
    name: String,
    active: String,
    items: Vec<(String, String, String)>,
    help: Help,
    javascript: String,
}

impl List {
    /// Construct dropdown list with a vector of body entries.
    pub fn with_names(name: &str, items: Vec<String>, help: &str) -> List {
        let items = items
            .into_iter()
            .map(|n| (n, "".to_string(), "".to_string()))
            .collect();
        let help = if help.is_empty() {
            Help::None
        } else {
            Help::Hardcoded(help.to_owned())
        };
        List {
            name: name.to_owned(),
            active: "".to_owned(),
            items,
            help,
            javascript: include_str!("../../files/list.js").to_owned(),
        }
    }

    /// Construct dropdown list with a vector of (body, help) entries. The help label is
    /// set to a help entry when that entry is selected.
    pub fn with_help(name: &str, items: Vec<(String, String)>) -> List {
        let items = items
            .into_iter()
            .map(|(n, h)| (n, h, "".to_string()))
            .collect();
        List {
            name: name.to_owned(),
            active: "".to_owned(),
            items,
            help: Help::PerItem,
            javascript: include_str!("../../files/list.js").to_owned(),
        }
    }

    /// Construct dropdown list with a vector of (body, custom_class) entries. The
    /// help label is set to a help entry when that entry is selected.
    pub fn with_class(name: &str, items: Vec<(String, String)>, help: &str) -> List {
        let items = items
            .into_iter()
            .map(|(n, c)| (n, "".to_string(), c))
            .collect();
        let help = if help.is_empty() {
            Help::None
        } else {
            Help::Hardcoded(help.to_owned())
        };
        List {
            name: name.to_owned(),
            active: "".to_owned(),
            items,
            help,
            javascript: include_str!("../../files/list.js").to_owned(),
        }
    }

    pub fn with_custom_js(self, javascript: &str) -> List {
        List {
            javascript: javascript.to_owned(),
            ..self
        }
    }

    pub fn without_js(self) -> List {
        List {
            javascript: "".to_owned(),
            ..self
        }
    }

    /// Name of the item to mark as selected on page load.
    pub fn with_active(self, active: &str) -> List {
        List {
            active: active.to_owned(),
            ..self
        }
    }
}

impl Widget for List {
    fn build(&self, builder: &mut EditorBuilder) {
        // list
        let mut list = HtmlTag::new("ul")
            .with_id("list") // we'll assume only one list on the page so we don't need to qualify the id
            .with_class("list-group")
            .with_attribute("aria-describedby", "list-help");
        for item in self.items.iter() {
            let mut entry = HtmlTag::new("li")
                .with_class("list-group-item")
                .with_attribute("onclick", "on_click(this)")
                .with_body(&item.0);
            if item.0 == self.active {
                entry.add_class("active");
                entry.add_attribute("aria-current", "true");
            }
            if !item.1.is_empty() {
                assert!(self.help == Help::PerItem);
                entry.add_attribute("data-help", &item.1);
            }
            if !item.2.is_empty() {
                entry.add_class(&item.2);
            }
            list.add_child(entry);
        }
        builder.form.add_child(list);

        // help
        match &self.help {
            Help::None => (),
            Help::PerItem => {
                let div = HtmlTag::new("div")
                    .with_id("list-help")
                    .with_class("form-text fst-italic fs-6 mb-4");
                builder.form.add_child(div);
            }
            Help::Hardcoded(s) => {
                let div = HtmlTag::new("div")
                    .with_id("list-help")
                    .with_class("form-text fst-italic fs-6 mb-4")
                    .with_body(&s);
                builder.form.add_child(div);
            }
        }

        // hidden button (used with post to send the selected item name)
        let button = HtmlTag::new("input")
            .with_id("list-button")
            .with_attribute("type", "text")
            .with_attribute("name", &self.name)
            .with_attribute("value", "")
            .with_attribute("hidden", "");
        builder.form.add_child(button);

        // javascript
        if !self.javascript.is_empty() {
            let js = HtmlTag::new("script")
                .with_attribute("type", "text/javascript")
                .with_body(&self.javascript);
            builder.prolog.push(js);
        }
    }
}

// =======================================================================================
/// List of radio buttons.
pub struct Radio {
    name: String,
    items: Vec<(String, String)>,
    help: String,
    checked: String,
}

impl Radio {
    /// Construct radios with a vector of (label, name) entries.
    pub fn new(name: &str, items: Vec<(String, String)>, help: &str) -> Radio {
        let items = items.into_iter().map(|(l, v)| (l, v)).collect();
        Radio {
            name: name.to_owned(),
            items,
            help: help.to_owned(),
            checked: "".to_owned(),
        }
    }

    /// Value of the item to mark as selected on page load.
    pub fn with_checked(self, checked: &str) -> Radio {
        Radio {
            checked: checked.to_owned(),
            ..self
        }
    }
}

impl Widget for Radio {
    fn build(&self, builder: &mut EditorBuilder) {
        // radios
        for item in self.items.iter() {
            let mut entry = HtmlTag::new("div").with_class("form-check");
            let mut input = HtmlTag::new("input")
                .with_id(&format!("{}-btn", item.1))
                .with_class("form-check-input")
                .with_attribute("type", "radio")
                .with_attribute("name", &self.name)
                .with_attribute("value", &item.1);
            if item.1 == self.checked {
                input.add_attribute("checked", "");
            }
            entry.add_child(input);

            let label = HtmlTag::new("label")
                .with_class("form-check-label")
                .with_attribute("for", &format!("{}-btn", item.1))
                .with_body(&item.0);
            entry.add_child(label);
            builder.form.add_child(entry);
        }

        // help
        let div = HtmlTag::new("div")
            .with_id(&format!("{}-help", self.name))
            .with_class("form-text fst-italic fs-6 mb-3")
            .with_body(&self.help);
        builder.form.add_child(div);
    }
}

// =======================================================================================
pub struct EditButton {
    id: String,
    onclick: String,
    body: String,
    attrs: HashMap<String, String>,
}

impl EditButton {
    pub fn new(id: &str, onclick: &str, body: &str) -> EditButton {
        EditButton {
            id: id.to_string(),
            onclick: onclick.to_string(),
            body: body.to_string(),
            attrs: HashMap::new(),
        }
    }

    pub fn with_attr(mut self, name: &str, value: &str) -> EditButton {
        self.attrs.insert(name.to_string(), value.to_string());
        self
    }
}

/// Construct these with the Prolog::with_* methods.
pub enum Prolog {
    Title(String),
    Editable(String, Vec<EditButton>, String),
}

impl Prolog {
    // Prolog has just a title.
    pub fn with_title(title: &str) -> Prolog {
        Prolog::Title(title.to_owned())
    }

    /// Prolog has a title and an edit menu populated with a slice of (id, onclick, body)
    /// entries.
    pub fn with_edit_menu(title: &str, buttons: Vec<EditButton>, javascript: &str) -> Prolog {
        Prolog::Editable(title.to_owned(), buttons, javascript.to_owned())
    }
}

impl Widget for Prolog {
    fn build(&self, builder: &mut EditorBuilder) {
        match self {
            Prolog::Title(title) => {
                let title = HtmlTag::new("h2")
                    .with_class("text-center")
                    .with_body(title);
                builder.prolog.push(title);
            }
            Prolog::Editable(title, items, javascript) => {
                // javascript
                let js = HtmlTag::new("script")
                    .with_attribute("type", "text/javascript")
                    .with_body(javascript);
                builder.prolog.push(js);

                let mut div = HtmlTag::new("div").with_class("d-flex");

                // helps center title
                let nbps = " ".repeat(if title.len() < 25 {
                    25 - title.len()
                } else {
                    1
                });
                let div2 = HtmlTag::new("div").with_class("p-1").with_body(&nbps);
                div.add_child(div2);

                let mut div2 = HtmlTag::new("div").with_class("p-1 flex-fill");
                div2.add_child(
                    HtmlTag::new("h2")
                        .with_class("text-center")
                        .with_body(&title),
                );
                div.add_child(div2);

                let mut div2 = HtmlTag::new("div").with_class("p-1 pe-2 justify-content-end");
                let mut div3 = HtmlTag::new("div").with_class("col justify-content-end");

                let mut div4 = HtmlTag::new("div").with_class("btn-group");
                let button = HtmlTag::new("button")
                    .with_class("btn btn-primary btn-sm dropdown-toggle")
                    .with_attribute("type", "dropdown")
                    .with_attribute("aria-expanded", "false")
                    .with_attribute("data-bs-toggle", "dropdown");
                div4.add_child(button);

                let mut ul = HtmlTag::new("ul").with_class("dropdown-menu");
                for item in items.iter() {
                    let mut li = HtmlTag::new("li");
                    let mut button = HtmlTag::new("button")
                        .with_id(&item.id)
                        .with_class("dropdown-item")
                        .with_attribute("type", "dropdown")
                        .with_body(&item.body);
                    if !item.onclick.is_empty() {
                        button.add_attribute("onclick", &item.onclick);
                    }
                    for (name, value) in item.attrs.iter() {
                        button.add_attribute(name, value);
                    }
                    li.add_child(button);
                    ul.add_child(li);
                }
                div4.add_child(ul);
                div3.add_child(div4);
                div2.add_child(div3);
                div.add_child(div2);

                builder.prolog.push(div);
            }
        }
    }
}

// =======================================================================================
/// The standard Cancel and Save buttons.
pub struct StdButtons {
    cancel_url: String,
    custom: Option<(String, String, String)>,
}

impl StdButtons {
    pub fn new(cancel_url: &str) -> StdButtons {
        StdButtons {
            cancel_url: cancel_url.to_owned(),
            custom: None,
        }
    }

    /// (label, class, post_url) appears between cancel and save buttons.
    pub fn with_custom(cancel_url: &str, custom: (&str, &str, &str)) -> StdButtons {
        let custom = Some((
            custom.0.to_string(),
            custom.1.to_string(),
            custom.2.to_string(),
        ));
        StdButtons {
            cancel_url: cancel_url.to_owned(),
            custom,
        }
    }
}

impl Widget for StdButtons {
    fn build(&self, builder: &mut EditorBuilder) {
        let mut div = HtmlTag::new("div").with_class("form-group mt-4");
        let mut div2 = HtmlTag::new("div").with_class("row justify-content-evenly");

        let mut div3 = HtmlTag::new("div").with_class("col-4 align-self-center");
        div3.add_child(
            HtmlTag::new("button")
                .with_class("btn btn-secondary")
                .with_body("Cancel")
                .with_attribute("type", "submit")
                .with_attribute("formmethod", "get")
                .with_attribute("formaction", &self.cancel_url)
                .with_attribute("formnovalidate", ""),
        );
        div2.add_child(div3);

        if let Some((label, class, post_url)) = &self.custom {
            let mut div3 = HtmlTag::new("div").with_class("col-4 align-self-center");
            div3.add_child(
                HtmlTag::new("button")
                    .with_class(&format!("btn {class}"))
                    .with_body("Cancel")
                    .with_attribute("type", "submit")
                    .with_attribute("formaction", &post_url)
                    .with_attribute("formnovalidate", "")
                    .with_body(&label),
            );
            div2.add_child(div3);
        }

        let mut div3 = HtmlTag::new("div").with_class("col-4 align-self-center");
        div3.add_child(
            HtmlTag::new("button")
                .with_class("btn btn-primary")
                .with_body("Save")
                .with_attribute("type", "submit"),
        );
        div2.add_child(div3);
        div.add_child(div2);
        builder.form.add_child(div);
    }
}

// =======================================================================================
/// Arbitrary multi-line text field.
pub struct TextArea {
    name: String,
    attrs: HashMap<String, String>,
    body: String,
    help: String,
}

impl TextArea {
    pub fn new(name: &str, rows: i32, cols: i32, help: &str) -> TextArea {
        let mut attrs = HashMap::new();
        attrs.insert("rows".to_string(), format!("{rows}"));
        attrs.insert("cols".to_string(), format!("{cols}"));

        TextArea {
            name: name.to_owned(),
            attrs,
            body: "".to_string(),
            help: help.to_owned(),
        }
    }

    pub fn with_body(self, body: &str) -> TextArea {
        TextArea {
            body: body.to_owned(),
            ..self
        }
    }

    pub fn with_spellcheck(mut self) -> TextArea {
        self.attrs
            .insert("spellcheck".to_string(), "true".to_string());
        self.attrs
            .insert("autocorrect".to_string(), "on".to_string());
        self
    }

    /// Kind can be sentences, words, or characters.
    pub fn with_autocapitalize(mut self, kind: &str) -> TextArea {
        self.attrs
            .insert("autocapitalize".to_string(), kind.to_string());
        self
    }

    // /// User has to enter something into the field in order to pass client side validation.
    // pub fn with_required(self) -> TextArea {
    // self.attrs
    //     .insert("required".to_string(), "".to_string());
    // self
    // }
}

impl Widget for TextArea {
    fn build(&self, builder: &mut EditorBuilder) {
        let key = self.name.to_lowercase();
        let key = key.replace(" ", "-");

        let input_id = format!("{key}-input");
        let help_id = format!("{key}-help");

        let mut div = HtmlTag::new("div").with_class("input-group");
        let mut text = HtmlTag::new("textarea")
            .with_id(&input_id)
            .with_class("form-control")
            .with_attribute("name", &self.name)
            .with_attribute("aria-describedby", &help_id)
            .with_body(&self.body);
        for (n, v) in self.attrs.iter() {
            text.add_attribute(&n, &v);
        }
        if !has_children(&builder.form) {
            text.add_attribute("autofocus", "");
        }
        div.add_child(text);
        builder.form.add_child(div);

        let help = HtmlTag::new("div")
            .with_id(&help_id)
            .with_class("form-text mb-4 fst-italic fs-6")
            .with_body(&self.help);
        builder.form.add_child(help);
    }
}

// =======================================================================================
/// Arbitrary one line text field.
pub struct TextInput {
    label: String,
    value: String,
    pattern: Option<String>,
    help: String,
    required: bool,
}

impl TextInput {
    pub fn new(label: &str, value: &str, help: &str) -> TextInput {
        TextInput {
            label: label.to_owned(),
            value: value.to_owned(),
            pattern: None,
            help: help.to_owned(),
            required: false,
        }
    }

    /// Add regex for client side validation.
    pub fn with_pattern(self, regex: &str) -> TextInput {
        TextInput {
            pattern: Some(regex.to_owned()),
            ..self
        }
    }

    /// User has to enter something into the field in order to pass client side validation.
    pub fn with_required(self) -> TextInput {
        TextInput {
            required: true,
            ..self
        }
    }
}

impl Widget for TextInput {
    fn build(&self, builder: &mut EditorBuilder) {
        let key = self.label.to_lowercase();
        let name = key.replace(" ", "_");
        let key = key.replace(" ", "-");

        let label_id = format!("{key}-label");
        let input_id = format!("{key}-input");
        let help_id = format!("{key}-help");

        let mut div = HtmlTag::new("div").with_class("input-group");
        let span = HtmlTag::new("span")
            .with_id(&label_id)
            .with_class("input-group-text")
            .with_body(&self.label);
        div.add_child(span);
        let mut input = HtmlTag::new("input")
            .with_id(&input_id)
            .with_class("form-control")
            .with_attribute("type", "text")
            .with_attribute("name", &name)
            .with_attribute("aria-describedby", &format!("{label_id} {help_id}"))
            .with_attribute("value", &self.value);
        if let Some(pattern) = &self.pattern {
            input.add_attribute("pattern", &pattern);
        }
        if self.required {
            input.add_attribute("required", "");
        }
        if !has_children(&builder.form) {
            input.add_attribute("autofocus", "");
        }
        div.add_child(input);
        builder.form.add_child(div);

        let help = HtmlTag::new("div")
            .with_id(&help_id)
            .with_class("form-text fst-italic fs-6 mb-4")
            .with_body(&self.help);
        builder.form.add_child(help);
    }
}

// =======================================================================================
fn has_children(tag: &HtmlTag) -> bool {
    if let Some(children) = &tag.children {
        !children.is_empty()
    } else {
        false
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
