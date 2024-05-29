use {regex::Regex, smartstring::alias::String, std::path::PathBuf};

#[derive(knuffel::Decode)]
pub struct Config {
    #[knuffel(child, unwrap(argument, str), default="".into())]
    pub link_base: String,
    #[knuffel(child, unwrap(argument, str), default="".into())]
    pub image_base: String,
    #[knuffel(children(name = "replace"))]
    pub replace: Vec<ReplaceRegex>,
    #[knuffel(children(name = "job"))]
    pub jobs: Vec<Job>,
}

#[derive(knuffel::Decode)]
pub struct ReplaceRegex {
    #[knuffel(argument, str)]
    pub from: Regex,
    #[knuffel(argument, str)]
    pub to: String,
}

#[derive(knuffel::Decode)]
pub struct Job {
    #[knuffel(argument, str)]
    pub from: PathBuf,
    #[knuffel(argument, str)]
    pub to: PathBuf,
    #[knuffel(children(name = "replace"))]
    pub replace: Vec<ReplaceRegex>,
    #[knuffel(children(name = "insert"))]
    pub insertions: Vec<Insertion>,
}

#[derive(knuffel::Decode)]
pub struct Insertion {
    #[knuffel(property, str)]
    pub before: Regex,
    #[knuffel(argument, str)]
    pub value: String,
}
