use anyhow::{anyhow, Result};
use std::io::{BufRead, BufReader, Read};
use std::iter::Iterator;

/// A comment must start with this prefix in order to be considered a directive
/// that we can handle.
const DIRECTIVE_PREFIX: &str = "mdbabel";

const CODEBLOCK_NAME_PARAMETER: &str = ":name";

/// Header that can be found in a code block directive.
#[derive(Debug, PartialEq)]
pub struct CodeBlockHeader {
    pub name: String,
}

impl CodeBlockHeader {
    /// Parse a code block header from the contents of a comment string.
    fn from_comment_contents(content: &str) -> Result<Self> {
        let mut ss = content.split_ascii_whitespace();

        match ss.next() {
            Some(s) => {
                if s != DIRECTIVE_PREFIX {
                    return Err(anyhow!("Header does not contain correct prefix"));
                }
            }
            None => return Err(anyhow!("Header contains nothing useful")),
        };

        // Name should always be the first parameter in the header.
        let name = match ss.next() {
            Some(CODEBLOCK_NAME_PARAMETER) => match ss.next() {
                Some(s) => s.to_owned(),
                None => return Err(anyhow!("No value for 'name' parameter")),
            },
            _ => return Err(anyhow!("Header doesn't contain 'name' as first parameter")),
        };

        Ok(CodeBlockHeader { name })
    }
}

/// The contents of a code block.
#[derive(Debug, PartialEq)]
pub struct CodeBlockBody {
    pub lang: Option<String>,
    pub code: String,
}

/// An 'mdbabel' directive parsed from a markdown document.
#[derive(Debug, PartialEq)]
pub enum Directive {
    /// A code block that should be executed.
    CodeBlock {
        header: CodeBlockHeader,
        body: CodeBlockBody,
    },
}

/// A markdown document to iterate over.
pub struct Document<R: Read> {
    reader: BufReader<R>,
    line_buf: String,
}

impl<R: Read> Document<R> {
    pub fn new(reader: R) -> Self {
        Document {
            reader: BufReader::new(reader),
            line_buf: String::new(),
        }
    }

    fn read_lines_while<P>(&mut self, predicate: P) -> Result<&str>
    where
        P: FnMut(&str) -> bool,
    {
        read_lines_while(&mut self.reader, &mut self.line_buf, predicate)
    }

    fn read_next_line(&mut self) -> Result<&str> {
        self.read_lines_while(|_| false)
    }
}

impl<R: Read> Iterator for Document<R> {
    type Item = Directive;

    fn next(&mut self) -> Option<Self::Item> {
        // Discard all lines that do not have a comment.
        let discard_pred = |line: &str| parse_comment_from_line(line).is_none();
        let line = self.read_lines_while(discard_pred).ok()?;
        let content = parse_comment_from_line(line)?;

        let header = match CodeBlockHeader::from_comment_contents(content) {
            Ok(header) => header,
            _ => return None,
        };

        // Read in starting code block delimeter immediately after the header.
        let line = self.read_next_line().ok()?;
        if !is_code_block_del(line) {
            return None;
        }
        let lang = parse_lang_from_code_block_del(line).map(|s| s.to_owned());

        // Collect all lines inside the code block.
        let mut code = String::new();
        let code_pred = |line: &str| {
            let is_code = !is_code_block_del(line);
            if is_code {
                code.push_str(line);
            }
            is_code
        };
        let _ = self.read_lines_while(code_pred).ok()?;

        let body = CodeBlockBody { lang, code };
        Some(Directive::CodeBlock { header, body })
    }
}

/// Read lines from the buffered reader while predicate keeps returning
/// true. The last read line will be returned.
fn read_lines_while<'a, R, P>(
    reader: &mut R,
    buf: &'a mut String,
    mut predicate: P,
) -> Result<&'a str>
where
    R: BufRead,
    P: FnMut(&str) -> bool,
{
    buf.truncate(0);
    let mut n = reader.read_line(buf)?;
    let mut line = &buf[0..n];
    while predicate(line) {
        buf.truncate(0);
        n = reader.read_line(buf)?;
        if n == 0 {
            return Err(anyhow!("End of file"));
        }
        line = &buf[0..n];
    }
    Ok(&buf[0..n])
}

/// If a line contains a comment, return the substring containing just the
/// comment contents (no delimeters). Leading and trailing whitespace will be
/// removed.
///
/// Comments should be in the form of '<!-- ... -->'.
fn parse_comment_from_line(line: &str) -> Option<&str> {
    let beg = line.find("<!--");
    let end = line.find("-->");
    match (beg, end) {
        (Some(beg_idx), Some(end_idx)) => {
            let content = &line[(beg_idx + 4)..(end_idx)].trim();
            Some(content)
        }
        _ => None,
    }
}

/// Parse the language from a code block delimeter.
fn parse_lang_from_code_block_del(line: &str) -> Option<&str> {
    match line.find("```") {
        Some(n) => {
            if line.len() > (n + 3) {
                Some(line[(n + 3)..].trim())
            } else {
                None
            }
        }
        None => None,
    }
}

fn is_code_block_del(line: &str) -> bool {
    line.starts_with("```")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_comment_from_line_basic() {
        let tests = vec![
            // Input, expected output.
            ("<!-- hello world -->", Some("hello world")),
            ("abc <!-- hello world -->", Some("hello world")),
            ("<!-- hello world --> dfg", Some("hello world")),
            ("<!---->", Some("")),
            ("# hello world", None),
        ];

        for test in tests {
            let input = test.0;
            let expected = test.1;
            let output = parse_comment_from_line(input);
            assert_eq!(expected, output);
        }
    }

    #[test]
    fn parse_lang_from_code_block_del_basic() {
        let tests = vec![
            // Input, expected output.
            ("```", None),
            ("```bash", Some("bash")),
            ("``` sh", Some("sh")),
        ];

        for test in tests {
            let input = test.0;
            let expected = test.1;
            let output = parse_lang_from_code_block_del(input);
            assert_eq!(expected, output);
        }
    }

    #[test]
    fn read_document_basic() {
        let content = "\
            # Document\n\
            \n\
            Some text\n\
            \n\
            <!-- mdbabel :name test-block -->\n\
            ```sh\n\
            echo 'hello world'\n\
            ```\n\
            \n\
            More text.\n";

        let mut doc = Document::new(content.as_bytes());

        let directive = doc.next().expect("expected code block directive");
        let expected = Directive::CodeBlock {
            header: CodeBlockHeader {
                name: "test-block".to_owned(),
            },
            body: CodeBlockBody {
                lang: Some("sh".to_owned()),
                code: "echo 'hello world'\n".to_owned(),
            },
        };
        assert_eq!(expected, directive);

        let next = doc.next();
        assert_eq!(None, next);
    }
}
