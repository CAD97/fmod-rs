use {
    self::config::Config,
    eyre::{eyre, WrapErr},
    html5ever::tendril::{StrTendril, TendrilSink},
    itertools::Itertools,
    markup5ever_rcdom::{Handle, NodeData, RcDom},
    std::{
        borrow::Cow,
        env, fs,
        path::{Path, PathBuf},
    },
};

mod config;
mod fmod2rustdoc;

pub fn main(fmod_path: &Path) -> color_eyre::Result<()> {
    let config_path = PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "..", "docgen.kdl"]);
    let config = knuffel::parse::<Config>(
        "docgen.kdl",
        &fs::read_to_string(&config_path).wrap_err_with(|| config_path.display().to_string())?,
    )
    .map_err(|err| eyre!("{:?}", miette::Report::new(err)))?;

    for job in &config.jobs {
        let html_path = fmod_path.join(&job.from);
        let html: RcDom = html5ever::parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .from_file(&html_path)
            .wrap_err_with(|| html_path.display().to_string())?;

        // NB: it's messy HTML, ignore the errors

        let (content, footer) =
            select_content(&html.document).wrap_err_with(|| html_path.display().to_string())?;

        let mut md = fmod2rustdoc::convert_all(content, &*footer, &config)?;

        for replace in &config.replace {
            if let Cow::Owned(x) = replace.from.replace_all(&*md, &*replace.to) {
                md = x;
            }
        }

        for replace in &job.replace {
            if let Cow::Owned(x) = replace.from.replace_all(&*md, &*replace.to) {
                md = x;
            }
        }

        let md_path =
            PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "..", "crates", "fmod-rs", "src"])
                .join(&job.to);
        fs::write(&md_path, md).wrap_err_with(|| md_path.display().to_string())?;
    }

    Ok(())
}

fn select_content(document: &Handle) -> color_eyre::Result<(Handle, StrTendril)> {
    assert!(matches!(document.data, NodeData::Document));

    let document_children = document.children.borrow();

    let html = document_children
        .iter()
        .filter(tag_is("html"))
        .at_most_one()
        .map_err(|_| eyre!("multiple <html>"))?
        .ok_or(eyre!("missing <html>"))?;

    let html_children = html.children.borrow();

    let body = html_children
        .iter()
        .filter(tag_is("body"))
        .at_most_one()
        .map_err(|_| eyre!("multiple <body>"))?
        .ok_or(eyre!("missing <body>"))?;

    let body_children = body.children.borrow();

    let docs_body = body_children
        .iter()
        .filter(tag_is("div"))
        .filter(class_is("docs-body"))
        .at_most_one()
        .map_err(|_| eyre!(r#"multiple <div class="docs-body">"#))?
        .ok_or(eyre!(r#"missing <div class="docs-body">"#))?;

    let docs_body_children = docs_body.children.borrow();

    let manual_content = docs_body_children
        .iter()
        .filter(tag_is("div"))
        .filter(class_is("manual-content api"))
        .at_most_one()
        .map_err(|_| eyre!(r#"multiple <div class="manual-content api">"#))?
        .ok_or(eyre!(r#"missing <div class="manual-content api">"#))?;

    let manual_footer = docs_body_children
        .iter()
        .filter(tag_is("p"))
        .filter(class_is("manual-footer"))
        .at_most_one()
        .map_err(|_| eyre!(r#"multiple <p class="manual-footer">"#))?
        .ok_or(eyre!(r#"missing <p class="manual-footer">"#))?;

    let manual_footer_children = manual_footer.children.borrow();

    let manual_footer_contents = manual_footer_children
        .iter()
        .at_most_one()
        .map_err(|_| eyre!(r#"<p class="manual-footer"> did not contain just text"#))?
        .map(|node| &node.data);

    let copyright = match manual_footer_contents {
        Some(NodeData::Text { contents }) => contents.borrow().clone(),
        None => Default::default(),
        _ => {
            return Err(eyre!(
                r#"<p class="manual-footer"> did not contain just text"#
            ))
        },
    };

    Ok((manual_content.clone(), copyright))
}

fn tag_is(tag: &str) -> impl '_ + Fn(&&Handle) -> bool {
    move |node| match &node.data {
        NodeData::Element { name, .. } => {
            let name = name.expanded();
            *name.ns == ns!(html) && name.local == tag
        },
        _ => false,
    }
}

fn class_is(class: &str) -> impl '_ + Fn(&&Handle) -> bool {
    |node| match &node.data {
        NodeData::Element { attrs, .. } => {
            let attrs = attrs.borrow();
            attrs.iter().any(|attr| {
                let name = attr.name.expanded();
                *name.ns == ns!() && name.local == "class" && *attr.value == *class
            })
        },
        _ => false,
    }
}
