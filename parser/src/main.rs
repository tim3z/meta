use pulldown_cmark::{Parser, *};
use serde_json::Value;
use std::ops::{Deref, Range};

fn main() {
    let filename = std::env::args().skip(1).next().expect("Need input file");
    let contents =
        std::fs::read_to_string(filename).expect("Something went wrong reading the file");

    let parser = Parser::new(&contents).into_offset_iter();

    let mut heading_stack = Vec::new();
    heading_stack.push(HeadingBuilder::new(0, 0..0));
    heading_stack.last_mut().unwrap().set_heading("file");
    let mut headings = Vec::new();
    let mut text_expectator: Option<Option<CowStr>> = None;

    let mut end_byte = 0;
    for (event, range) in parser {
        dbg!(&event, &range);
        end_byte = range.end;

        match event {
            Event::Start(Tag::Heading(level)) => {
                while heading_stack.last().unwrap().level >= level {
                    headings.push(heading_stack.pop().unwrap().finalize(range.start));
                }
                heading_stack
                    .last_mut()
                    .unwrap()
                    .end_own_content(range.start);
                heading_stack.push(HeadingBuilder::new(level, range));
                text_expectator = Some(None);
            }
            Event::End(Tag::Heading(level)) => {
                debug_assert_eq!(heading_stack.last().unwrap().level, level);
                heading_stack
                    .last_mut()
                    .unwrap()
                    .set_heading(&text_expectator.take().unwrap().unwrap());
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if "json meta" == lang.deref() {
                    text_expectator = Some(None);
                }
            }
            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if "json meta" == lang.deref() {
                    heading_stack
                        .last_mut()
                        .unwrap()
                        .add_meta(&text_expectator.take().unwrap().unwrap());
                }
            }
            Event::Text(title) => {
                if let Some(expectator) = text_expectator.as_mut() {
                    debug_assert!(expectator.is_none());
                    *expectator = Some(title);
                }
            }
            _ => (),
        }
    }
    while let Some(heading_builder) = heading_stack.pop() {
        headings.push(heading_builder.finalize(end_byte))
    }

    dbg!(headings);
}

struct HeadingBuilder {
    level: u32,
    title: Option<String>,
    meta: Value,
    heading_range: Range<usize>,
    content_end: Option<usize>,
}

impl HeadingBuilder {
    fn new(level: u32, heading_range: Range<usize>) -> Self {
        Self {
            level,
            content_end: None,
            heading_range,
            meta: Value::Null,
            title: None,
        }
    }

    fn set_heading(&mut self, title: &str) {
        self.title = Some(title.to_string());
    }

    fn add_meta(&mut self, data: &str) {
        self.meta = serde_json::from_str(data).unwrap();
    }

    fn end_own_content(&mut self, end_byte: usize) {
        if self.content_end.is_none() {
            self.content_end = Some(end_byte);
        }
    }

    fn finalize(self, end_byte: usize) -> Heading {
        Heading {
            title: self.title.unwrap(),
            meta: self.meta,
            line: 0,
            content_end: self.content_end.unwrap_or(self.heading_range.end),
            children_content_end: end_byte,
            heading_range: self.heading_range,
        }
    }
}

#[derive(Debug)]
struct Heading {
    title: String,
    meta: Value,
    line: usize,
    heading_range: Range<usize>,
    content_end: usize,
    children_content_end: usize,
    // file: PathBuf,
    // accessed_at: SystemTime,
}

// TODO merge meta stuff
// TODO remember parents
