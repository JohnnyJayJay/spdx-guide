// Copyright (C) 2022  JohnnyJayJay

#[cfg(windows)]
const LINE_ENDING : &str = "\r\n";
#[cfg(not(windows))]
const LINE_ENDING : &str = "\n";

#[derive(Debug)]
pub struct SpdxEntry {
    tag: String,
    value: String,
}

impl SpdxEntry {
    fn new(tag: String, value: String) -> Self {
        SpdxEntry {
            tag, value
        }
    }
}

#[derive(Debug)]
pub enum SpdxLine {
    Empty,
    Comment(String),
    Entry(SpdxEntry),
}

#[derive(Default, Debug)]
pub struct SpdxSection {
    lines: Vec<SpdxLine>
}

impl SpdxSection {
    pub fn add_entry<T: Into<String>, V: Into<String>>(&mut self, tag: T, value: V) {
        self.lines.push(SpdxLine::Entry(SpdxEntry::new(tag.into(), value.into())));
    }

    pub fn add_comment<T: Into<String>>(&mut self, comment: T) {
        self.lines.push(SpdxLine::Comment(comment.into()));
    }

    fn value_with_tag<'a>(&self, line: &'a SpdxLine, tag: &str) -> Option<&'a str> {
        if let SpdxLine::Entry(SpdxEntry { tag: found_tag, value }) = line {
            if tag == found_tag {
                Some(value.as_str())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn find(&self, tag: &str) -> Vec<&str> {
        (&self.lines).into_iter()
            .filter_map(|line| self.value_with_tag(line, tag))
            .collect()
    }

}

impl ToString for SpdxSection {
    fn to_string(&self) -> String {
        let mut str = String::new();
        for line in &self.lines {
            match line {
                SpdxLine::Empty => {},
                SpdxLine::Comment(comment) => {
                    str.push('#');
                    str.push_str(comment);
                }
                SpdxLine::Entry(SpdxEntry { tag, value }) => {
                    str.push_str(tag);
                    str.push_str(": ");
                    str.push_str(value);
                }
            }
            str.push_str(LINE_ENDING);
        }
        str
    }

}

#[derive(Default, Debug)]
pub struct SpdxDocument {
    pub document_section: SpdxSection,
    pub package_section: SpdxSection,
}

impl ToString for SpdxDocument {
    fn to_string(&self) -> String {
        let mut str = String::new();
        str.push_str("##### Document Information");
        str.push_str(LINE_ENDING);
        str.push_str(&self.document_section.to_string());
        str.push_str(&LINE_ENDING.repeat(2));
        str.push_str("##### Package Information");
        str.push_str(LINE_ENDING);
        str.push_str(&self.package_section.to_string());
        str
    }
}