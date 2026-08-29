use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

pub fn markdown_to_pango(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(markdown, options);
    let mut pango = String::new();
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut _code_block_lang = String::new();
    let mut code_block_buf = String::new();
    let mut quote_depth = 0;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Paragraph => {}
                Tag::Heading { level, .. } => {
                    let (size, color) = match level {
                        HeadingLevel::H1 => ("xx-large", " color=\"#3584e4\""),
                        HeadingLevel::H2 => ("x-large", " color=\"#1c71d8\""),
                        HeadingLevel::H3 => ("large", " color=\"#62a0ea\""),
                        HeadingLevel::H4 => ("medium", ""),
                        HeadingLevel::H5 => ("small", ""),
                        HeadingLevel::H6 => ("x-small", ""),
                    };
                    pango.push_str(&format!("<span size=\"{}\" weight=\"bold\"{}>", size, color));
                }
                Tag::BlockQuote(_) => {
                    quote_depth += 1;
                    pango.push_str("<span style=\"italic\" alpha=\"80%\">");
                    pango.push_str("▎ ");
                }
                Tag::CodeBlock(kind) => {
                    in_code_block = true;
                    _code_block_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    code_block_buf.clear();
                }
                Tag::List(_) => {
                    list_depth += 1;
                }
                Tag::Item => {
                    let indent = "  ".repeat(list_depth.saturating_sub(1));
                    pango.push_str(&format!("{}• ", indent));
                }
                Tag::Emphasis => {
                    pango.push_str("<i>");
                }
                Tag::Strong => {
                    pango.push_str("<b>");
                }
                Tag::Strikethrough => {
                    pango.push_str("<s>");
                }
                Tag::Link { dest_url, .. } => {
                    pango.push_str(&format!("<a href=\"{}\"><u>", escape_xml(&dest_url)));
                }
                Tag::Image { dest_url, .. } => {
                    // Render images as a clickable link to the source file,
                    // since a plain GTK label cannot display bitmap content.
                    pango.push_str(&format!(
                        "<span foreground=\"#3584e4\" underline=\"single\">🖼 image: {}</span>",
                        escape_xml(&dest_url)
                    ));
                }
                Tag::Table(_) => {
                    pango.push_str("<tt>");
                }
                Tag::TableRow => {}
                Tag::TableCell => {
                    pango.push_str(" | ");
                }
                _ => {}
            },
            Event::End(tag) => match tag {
                TagEnd::Paragraph => {
                    pango.push_str("\n\n");
                }
                TagEnd::Heading(_) => {
                    pango.push_str("</span>\n\n");
                }
                TagEnd::BlockQuote(_) => {
                    pango.push_str("</span>\n\n");
                    if quote_depth > 0 {
                        quote_depth -= 1;
                    }
                }
                TagEnd::CodeBlock => {
                    in_code_block = false;
                    let escaped = escape_xml(&code_block_buf);
                    pango.push_str(&format!(
                        "\n<tt><span background=\"#1e1e1e\" foreground=\"#dcdcdc\">{}</span></tt>\n\n",
                        escaped
                    ));
                    code_block_buf.clear();
                }
                TagEnd::List(_) => {
                    pango.push('\n');
                    list_depth = list_depth.saturating_sub(1);
                }
                TagEnd::Item => {
                    pango.push('\n');
                }
                TagEnd::Emphasis => {
                    pango.push_str("</i>");
                }
                TagEnd::Strong => {
                    pango.push_str("</b>");
                }
                TagEnd::Strikethrough => {
                    pango.push_str("</s>");
                }
                TagEnd::Link => {
                    pango.push_str("</u></a>");
                }
                TagEnd::Table => {
                    pango.push_str("</tt>\n\n");
                }
                TagEnd::TableRow => {
                    pango.push_str(" |\n");
                }
                TagEnd::TableCell => {}
                _ => {}
            },
            Event::Text(text) => {
                if in_code_block {
                    code_block_buf.push_str(&text);
                } else {
                    pango.push_str(&escape_xml(&text));
                }
            }
            Event::Code(code) => {
                pango.push_str(&format!(
                    "<tt><span background=\"#282828\" foreground=\"#33d17a\"> {} </span></tt>",
                    escape_xml(&code)
                ));
            }
            Event::TaskListMarker(checked) => {
                if checked {
                    pango.push_str("<span foreground=\"#33d17a\" weight=\"bold\">[☑] </span>");
                } else {
                    pango.push_str("<span foreground=\"#888888\" weight=\"bold\">[☐] </span>");
                }
            }
            Event::SoftBreak => {
                pango.push('\n');
            }
            Event::HardBreak => {
                pango.push_str("\n\n");
            }
            Event::Rule => {
                pango.push_str("<span alpha=\"50%\">────────────────────────────────────────────</span>\n\n");
            }
            _ => {}
        }
    }

    pango.trim_end().to_string()
}

fn escape_xml(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_formatting() {
        let md = "# Title\n\nThis is **bold** and *italic*.\n\n- [x] Done\n- [ ] Todo";
        let pango = markdown_to_pango(md);
        assert!(pango.contains("size=\"xx-large\""));
        assert!(pango.contains("<b>bold</b>"));
        assert!(pango.contains("<i>italic</i>"));
        assert!(pango.contains("[☑]"));
        assert!(pango.contains("[☐]"));
    }
}
