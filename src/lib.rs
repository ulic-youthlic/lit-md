use pulldown_cmark::{
    html::push_html, CodeBlockKind, CowStr, Event, Options, Parser as MDParser, Tag, TagEnd,
    TextMergeStream,
};
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, BufReader, Error, ErrorKind},
    path::PathBuf,
};

mod highlight;

pub struct HtmlMaker {
    reader: BufReader<File>,
}

impl TryFrom<PathBuf> for HtmlMaker {
    type Error = Error;
    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        if "md"
            != value.extension().ok_or(Self::Error::new(
                ErrorKind::Other,
                "Con NOT get file extension.",
            ))?
        {
            return Err(Self::Error::new(
                ErrorKind::InvalidInput,
                "Can not open non-md file.",
            ));
        };
        let file = OpenOptions::new().read(true).open(value)?;
        let reader = BufReader::new(file);
        Ok(Self { reader })
    }
}

markup::define! {
    Code <'s, 'code_type, 'code>(s: &'s str, code_type:&'code Option<CowStr<'code_type>>){
        @markup::raw(highlight::code_to_html(s, code_type))
    }
}

impl HtmlMaker {
    pub fn parse(&mut self) -> Result<String, Error> {
        let mut content = String::new();
        self.reader.read_to_string(&mut content)?;

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let md_parser = TextMergeStream::new(MDParser::new_ext(&content, options));
        let mut in_code_block = false;
        let mut code_type = None;
        let md_parser = md_parser.filter_map(|event| match event {
            Event::Start(Tag::CodeBlock(name)) => {
                in_code_block = true;
                match name {
                    CodeBlockKind::Indented => (),
                    CodeBlockKind::Fenced(lang) => code_type = Some(lang),
                }
                None
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                code_type = None;
                None
            }
            Event::Text(s) => {
                if in_code_block {
                    Some(Event::InlineHtml(
                        Code {
                            code_type: &code_type,
                            s: s.as_ref(),
                        }
                        .to_string()
                        .into(),
                    ))
                } else {
                    Some(Event::Text(s))
                }
            }
            _ => Some(event),
        });
        let mut html = String::new();
        push_html(&mut html, md_parser);
        Ok(html)
    }
}
