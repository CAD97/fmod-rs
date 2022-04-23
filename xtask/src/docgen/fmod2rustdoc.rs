use {
    super::config::Config,
    color_eyre::Result,
    eyre::eyre,
    html5ever::{tendril::StrTendril, Attribute},
    itertools::Itertools,
    markup5ever_rcdom::{Handle, NodeData},
    once_cell::sync::OnceCell,
    regex::Regex,
    std::{fmt::Write, iter},
};

struct Converter<'a> {
    out: &'a mut String,
    ordered: bool,
    config: &'a Config,
}

impl<'a> Converter<'a> {
    pub fn new(out: &'a mut String, config: &'a Config) -> Self {
        Self {
            out,
            ordered: false,
            config,
        }
    }
}

pub fn convert_all(html: Handle, source: &str, config: &Config) -> Result<String> {
    let mut out = String::new();
    let mut converter = Converter::new(&mut out, config);

    let children = html.children.borrow();
    let mut children = children.iter();

    for child in children.by_ref() {
        if converter.convert_first(child)? {
            break;
        }
    }

    writeln!(&mut converter.out, "(This document is from the {source})\n").unwrap();

    for child in children {
        // let _ = converter.convert(child);
        converter.convert(child)?;
    }

    Ok(out)
}

fn re_whitespace() -> &'static Regex {
    static ONCE: OnceCell<Regex> = OnceCell::new();
    ONCE.get_or_init(|| Regex::new(r"\s+").unwrap())
}

impl Converter<'_> {
    fn convert_first(&mut self, node: &Handle) -> Result<bool> {
        match &node.data {
            NodeData::Text { contents } => {
                let contents = contents.borrow();
                if contents.is_empty() || re_whitespace().is_match(&**contents) {
                    return Ok(false);
                }
            },
            NodeData::Comment { .. } => return Ok(false),
            _ => (),
        }

        self.convert(node)?;
        Ok(true)
    }

    fn convert(&mut self, node: &Handle) -> Result<()> {
        match &node.data {
            NodeData::Document => unreachable!(),
            NodeData::Doctype { .. } => Err(eyre!("unexpected <!DOCTYPE>")),
            NodeData::Text { contents } => {
                let contents = contents.borrow();
                // Normalize HTML whitespace
                self.out
                    .push_str(&*re_whitespace().replace_all(&**contents, " "));
                Ok(())
            },
            NodeData::Comment { .. } => Ok(()),
            NodeData::Element { name, attrs, .. } => {
                let name = name.expanded();
                assert_eq!(*name.ns, ns!(html));
                let name = name.local;
                let children = node.children.borrow();
                let attrs = attrs.borrow();

                // NB: this is super hacky and just barely works. However, the self.output is spot-checked,
                // so this is okay, so long as this gets maintained. If we're generating bad MD, this
                // is the place to check!
                match *name {
                    local_name!("h1") => {
                        self.out.push_str("# ");
                        for child in &**children {
                            self.convert(&strip_a(child))?;
                        }
                        self.out.push_str("\n\n");
                        Ok(())
                    },
                    local_name!("h2") => {
                        self.out.push_str("## ");
                        for child in &**children {
                            self.convert(&strip_a(child))?;
                        }
                        self.out.push_str("\n\n");
                        Ok(())
                    },
                    local_name!("h3") => {
                        self.out.push_str("### ");
                        for child in &**children {
                            self.convert(&strip_a(child))?;
                        }
                        self.out.push_str("\n\n");
                        Ok(())
                    },
                    local_name!("h4") => {
                        self.out.push_str("#### ");
                        for child in &**children {
                            self.convert(&strip_a(child))?;
                        }
                        self.out.push_str("\n\n");
                        Ok(())
                    },
                    local_name!("p") => {
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push_str("\n\n");
                        Ok(())
                    },
                    local_name!("em") => {
                        self.out.push('*');
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push('*');
                        Ok(())
                    },
                    local_name!("strong") => {
                        self.out.push_str("**");
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push_str("**");
                        Ok(())
                    },
                    local_name!("br") => {
                        self.out.push_str("  \n");
                        Ok(())
                    },
                    local_name!("code") => {
                        self.out.push('`');
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push('`');
                        Ok(())
                    },
                    local_name!("ul") => {
                        self.ordered = false;
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push('\n');
                        Ok(())
                    },
                    local_name!("ol") => {
                        self.ordered = true;
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push('\n');
                        Ok(())
                    },
                    local_name!("li") => {
                        match self.ordered {
                            true => self.out.push_str("1. "),
                            false => self.out.push_str("- "),
                        }
                        for child in &**children {
                            self.convert(child)?;
                        }
                        self.out.push('\n');
                        Ok(())
                    },
                    local_name!("img") => {
                        let image_base = &self.config.image_base;
                        let alt = get_attr(&attrs, "alt").unwrap_or_default();
                        let src = get_attr(&attrs, "src").unwrap_or_default();
                        write!(self.out, "![{alt}]({image_base}{src})")?;
                        Ok(())
                    },
                    local_name!("div") => {
                        let class = get_attr(&attrs, "class").unwrap_or_default();
                        match &*class {
                            "highlight language-text" => {
                                let text = get_text_transitively(children.iter().cloned());
                                self.out.push_str("``````````text\n");
                                for child in text {
                                    self.out.push_str(&*child);
                                }
                                self.out.push_str("``````````\n\n");
                                Ok(())
                            },
                            "mixdowntable" => {
                                for child in children.iter() {
                                    self.convert(child)?;
                                }
                                Ok(())
                            },
                            _ => Err(eyre!(r#"unhandled element <{name} class="{class}">"#)),
                        }
                    },
                    local_name!("a") => {
                        let class = get_attr(&attrs, "class").unwrap_or_default();
                        let title = get_attr(&attrs, "title");
                        let href = get_attr(&attrs, "href").unwrap_or_default();

                        if *class == *"apilink" {
                            if let Ok(Some(child)) = children.iter().at_most_one() {
                                let mut api_name = String::new();
                                Converter::new(&mut api_name, self.config).convert(child)?;
                                if let Some(title) = title {
                                    write!(self.out, r#"[`{api_name}`]({api_name} "{title}")"#)?;
                                } else {
                                    write!(self.out, "[`{api_name}`]({api_name})")?;
                                }
                            } else {
                                return Err(eyre!(
                                    r#"unexpected <a class="apilink"> with complex children"#
                                ));
                            }
                        } else {
                            let mut link_text = String::new();
                            let link_base = &self.config.link_base;
                            let mut converter = Converter::new(&mut link_text, self.config);

                            for child in &**children {
                                converter.convert(child)?;
                            }
                            if let Some(title) = title {
                                write!(
                                    self.out,
                                    r#"[{link_text}](<{link_base}{href}> "{title}")"#
                                )?;
                            } else {
                                write!(self.out, "[{link_text}](<{link_base}{href}>)")?;
                            }
                        }
                        Ok(())
                    },
                    local_name!("table") => {
                        let thead = children
                            .iter()
                            .filter(super::tag_is("thead"))
                            .at_most_one()
                            .map_err(|_| eyre!(r#"multiple <thead> in <table>"#))?
                            .ok_or(eyre!(r#"missing <thead> in <table>"#))?;

                        let tbody = children
                            .iter()
                            .filter(super::tag_is("tbody"))
                            .at_most_one()
                            .map_err(|_| eyre!(r#"multiple <tbody> in <table>"#))?
                            .ok_or(eyre!(r#"missing <tbody> in <table>"#))?;

                        self.convert_table(thead, tbody)
                    },
                    _ => Err(eyre!("unhandled element <{name}>")),
                }
            },
            NodeData::ProcessingInstruction { .. } => {
                Err(eyre!("unhandled processing instruction"))
            },
        }
    }

    fn convert_table(&mut self, head: &Handle, body: &Handle) -> Result<()> {
        let head_children = head.children.borrow();
        let head_tr = head_children
            .iter()
            .filter(super::tag_is("tr"))
            .at_most_one()
            .map_err(|_| eyre!(r#"multiple <tr> in <thead>"#))?
            .ok_or(eyre!(r#"missing <tr> in <thead>"#))?;
        let head_tr_children = head_tr.children.borrow();
        let ths = head_tr_children
            .iter()
            .filter(super::tag_is("th"))
            .collect::<Vec<_>>();

        let table_width = ths.len();

        for th in ths {
            self.out.push_str("| ");
            for child in th.children.borrow().iter() {
                self.convert(child)?;
            }
            self.out.push(' ');
        }
        self.out.push_str("|\n");
        for _ in 0..table_width {
            self.out.push_str("| :-: ")
        }
        self.out.push_str("|\n");

        let body_children = body.children.borrow();
        let trs = body_children.iter().filter(super::tag_is("tr"));

        for tr in trs {
            let tr_children = tr.children.borrow();
            let tds = tr_children.iter().filter(super::tag_is("td"));

            for td in tds {
                self.out.push_str("| ");
                for child in td.children.borrow().iter() {
                    self.convert(child)?;
                }
                self.out.push(' ');
            }
            self.out.push_str("|\n");
        }

        self.out.push('\n');
        Ok(())
    }
}

fn get_attr(attrs: &[Attribute], pick: &str) -> Option<StrTendril> {
    attrs.iter().find_map(|attr| {
        let name = attr.name.expanded();
        if *name.ns == ns!() && *name.local == *pick {
            Some(attr.value.clone())
        } else {
            None
        }
    })
}

fn get_text_transitively<'a>(
    children: impl 'a + Iterator<Item = Handle>,
) -> Box<dyn 'a + Iterator<Item = StrTendril>> {
    let result = children.into_iter().flat_map(move |handle| {
        let self_text = match &handle.data {
            NodeData::Text { contents } => Some(contents.borrow().clone()),
            _ => None,
        };

        let child_text = handle
            .children
            .borrow()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .map(iter::once)
            .flat_map(get_text_transitively);

        self_text.into_iter().chain(child_text)
    });

    Box::new(result)
}

fn strip_a(node: &Handle) -> Handle {
    if super::tag_is("a")(&node) {
        node.children.borrow()[0].clone()
    } else {
        node.clone()
    }
}

/// This is not proper HTML serialization, of course.
fn _dump_html(indent: usize, handle: &Handle) {
    let node = handle;
    // FIXME: don't allocate
    print!("{}", " ".repeat(indent));
    match node.data {
        NodeData::Document => println!("#Document"),

        NodeData::Doctype {
            ref name,
            ref public_id,
            ref system_id,
        } => println!("<!DOCTYPE {} \"{}\" \"{}\">", name, public_id, system_id),

        NodeData::Text { ref contents } => {
            println!("#text: {}", contents.borrow().escape_default())
        },

        NodeData::Comment { ref contents } => println!("<!-- {} -->", contents.escape_default()),

        NodeData::Element {
            ref name,
            ref attrs,
            ..
        } => {
            assert!(name.ns == ns!(html));
            print!("<{}", name.local);
            for attr in attrs.borrow().iter() {
                assert!(attr.name.ns == ns!());
                print!(" {}=\"{}\"", attr.name.local, attr.value);
            }
            println!(">");
        },

        NodeData::ProcessingInstruction { .. } => unreachable!(),
    }

    for child in node.children.borrow().iter() {
        _dump_html(indent + 4, child);
    }
}
