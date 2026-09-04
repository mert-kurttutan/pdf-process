//! A deliberately conservative HTML-to-Typst mapper.
//!
//! Math is kept as source text inside Typst math delimiters rather than being
//! rebuilt from PDF glyph positions.

#[derive(Default)]
struct HtmlToTypstParser {
    chunks: Vec<String>,
    list_depth: usize,
    skipped_tags: Vec<String>,
    math_tags: Vec<String>,
}

impl HtmlToTypstParser {
    fn start_tag(&mut self, tag: &str, class: Option<&str>) {
        if matches!(tag, "script" | "style" | "head") {
            self.skipped_tags.push(tag.to_owned());
            return;
        }
        if !self.skipped_tags.is_empty() {
            return;
        }
        if tag == "math"
            || class
                .is_some_and(|classes| classes.split_ascii_whitespace().any(|item| item == "math"))
        {
            self.math_tags.push(tag.to_owned());
            self.append("$ ");
            return;
        }
        match tag {
            "p" | "div" | "section" | "article" | "tr" => self.blank_line(),
            "br" => self.append("\n"),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.blank_line();
                let level = tag[1..].parse::<usize>().unwrap_or(1);
                self.append(&format!("{} ", "=".repeat(level)));
            }
            "strong" | "b" => self.append("*"),
            "em" | "i" => self.append("_"),
            "ul" => {
                self.list_depth += 1;
                self.blank_line();
            }
            "li" => self.append(&format!(
                "\n{}- ",
                "  ".repeat(self.list_depth.saturating_sub(1))
            )),
            "td" | "th" => self.append(" | "),
            _ => {}
        }
    }

    fn end_tag(&mut self, tag: &str) {
        if self
            .skipped_tags
            .last()
            .is_some_and(|open_tag| open_tag == tag)
        {
            self.skipped_tags.pop();
            return;
        }
        if !self.skipped_tags.is_empty() {
            return;
        }
        if self
            .math_tags
            .last()
            .is_some_and(|open_tag| open_tag == tag)
        {
            self.append(" $");
            self.math_tags.pop();
            return;
        }
        match tag {
            "p" | "div" | "section" | "article" | "tr" | "table" | "h1" | "h2" | "h3" | "h4"
            | "h5" | "h6" => self.blank_line(),
            "strong" | "b" => self.append("*"),
            "em" | "i" => self.append("_"),
            "ul" => {
                self.list_depth = self.list_depth.saturating_sub(1);
                self.blank_line();
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        if !self.skipped_tags.is_empty() {
            return;
        }
        let text = collapse_whitespace(&decode_entities(text));
        if text.is_empty() {
            return;
        }
        if self.math_tags.is_empty() {
            self.append(&escape_typst_text(&text));
        } else {
            self.append(&text);
        }
    }

    fn append(&mut self, text: &str) {
        let no_space_before = matches!(
            text.chars().next(),
            Some('\n' | ' ' | '.' | ',' | ':' | ';' | ')' | ']' | '}')
        );
        if let Some(previous) = self.chunks.last()
            && !previous.is_empty()
            && !previous.ends_with('\n')
            && !previous.ends_with(' ')
            && !previous.ends_with('|')
            && !text.is_empty()
            && !no_space_before
        {
            self.chunks.push(" ".to_owned());
        }
        self.chunks.push(text.to_owned());
    }

    fn blank_line(&mut self) {
        if self.chunks.is_empty() {
            return;
        }
        let content = self.chunks.concat();
        if !content.ends_with("\n\n") {
            self.chunks.push("\n\n".to_owned());
        }
    }

    fn render(self) -> String {
        let content = self.chunks.concat();
        let mut compact = Vec::new();
        let mut previous_blank = true;
        for line in content.lines() {
            let line = line.trim_end();
            let is_blank = line.is_empty();
            if !(is_blank && previous_blank) {
                compact.push(line);
            }
            previous_blank = is_blank;
        }
        let body = compact.join("\n").trim().to_owned();
        format!("#set page(margin: 1in)\n#set text(size: 11pt)\n\n{body}\n")
    }
}

/// Convert common structural HTML into baseline Typst source.
#[must_use]
pub fn html_to_typst(html: &str) -> String {
    let mut parser = HtmlToTypstParser::default();
    let mut cursor = 0;
    while cursor < html.len() {
        let remainder = &html[cursor..];
        let Some(open_offset) = remainder.find('<') else {
            parser.text(remainder);
            break;
        };
        parser.text(&remainder[..open_offset]);
        cursor += open_offset;
        let remainder = &html[cursor..];
        if remainder.starts_with("<!--") {
            cursor += remainder.find("-->").map_or(remainder.len(), |end| end + 3);
            continue;
        }
        let Some(tag_end) = find_tag_end(remainder) else {
            parser.text(remainder);
            break;
        };
        parse_tag(&remainder[1..tag_end], &mut parser);
        cursor += tag_end + 1;
    }
    parser.render()
}

fn find_tag_end(input: &str) -> Option<usize> {
    let mut quote = None;
    for (index, character) in input.char_indices().skip(1) {
        match (quote, character) {
            (None, '\'' | '"') => quote = Some(character),
            (Some(active), character) if active == character => quote = None,
            (None, '>') => return Some(index),
            _ => {}
        }
    }
    None
}

fn parse_tag(raw_tag: &str, parser: &mut HtmlToTypstParser) {
    let raw_tag = raw_tag.trim();
    if raw_tag.is_empty() || raw_tag.starts_with('!') || raw_tag.starts_with('?') {
        return;
    }
    let closing = raw_tag.starts_with('/');
    let content = raw_tag.trim_start_matches('/').trim();
    let tag = content
        .split(|character: char| character.is_whitespace() || character == '/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if tag.is_empty() {
        return;
    }
    if closing {
        parser.end_tag(&tag);
    } else {
        let class = attribute_value(content, "class");
        parser.start_tag(&tag, class.as_deref());
        if raw_tag.ends_with('/') {
            parser.end_tag(&tag);
        }
    }
}

fn attribute_value(content: &str, target: &str) -> Option<String> {
    let mut cursor = content.find(char::is_whitespace).unwrap_or(content.len());
    let bytes = content.as_bytes();
    while cursor < bytes.len() {
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < bytes.len()
            && !bytes[cursor].is_ascii_whitespace()
            && bytes[cursor] != b'='
            && bytes[cursor] != b'/'
        {
            cursor += 1;
        }
        if name_start == cursor {
            cursor += 1;
            continue;
        }
        let name = &content[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != b'=' {
            continue;
        }
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let (start, end) = if cursor < bytes.len() && matches!(bytes[cursor], b'\'' | b'"') {
            let quote = bytes[cursor];
            cursor += 1;
            let start = cursor;
            while cursor < bytes.len() && bytes[cursor] != quote {
                cursor += 1;
            }
            let end = cursor;
            if cursor < bytes.len() {
                cursor += 1;
            }
            (start, end)
        } else {
            let start = cursor;
            while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            (start, cursor)
        };
        if name.eq_ignore_ascii_case(target) {
            return Some(decode_entities(&content[start..end]));
        }
    }
    None
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_entities(text: &str) -> String {
    let mut decoded = text
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    while let Some(start) = decoded.find("&#") {
        let Some(end_relative) = decoded[start..].find(';') else {
            break;
        };
        let end = start + end_relative;
        let entity = &decoded[start + 2..end];
        let codepoint = entity
            .strip_prefix('x')
            .or_else(|| entity.strip_prefix('X'))
            .and_then(|number| u32::from_str_radix(number, 16).ok())
            .or_else(|| entity.parse::<u32>().ok());
        let Some(character) = codepoint.and_then(char::from_u32) else {
            break;
        };
        decoded.replace_range(start..=end, &character.to_string());
    }
    decoded
}

fn escape_typst_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('#', "\\#")
        .replace('$', "\\$")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('[', "\\[")
        .replace(']', "\\]")
}

#[cfg(test)]
mod tests {
    use super::html_to_typst;

    #[test]
    fn maps_headings_paragraphs_and_math() {
        let typst =
            html_to_typst("<h1>Title</h1><p>Area</p><p><span class='math'>A = pi r^2</span></p>");
        assert!(typst.contains("= Title"));
        assert!(typst.contains("Area"));
        assert!(typst.contains("$ A = pi r^2 $"));
    }

    #[test]
    fn escapes_typst_control_characters() {
        let typst = html_to_typst("<p>#total * [x]</p>");
        assert!(typst.contains("\\#total \\* \\[x\\]"));
    }

    #[test]
    fn preserves_nested_math_as_one_expression() {
        let typst = html_to_typst("<span class=\"math\">x<sup>2</sup> + y</span>");
        assert!(typst.contains("$ x 2 + y $"));
    }
}
