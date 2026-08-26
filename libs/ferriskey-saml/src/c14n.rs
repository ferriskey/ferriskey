use std::collections::{BTreeMap, BTreeSet};

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum C14nError {
    #[error("malformed xml: {0}")]
    MalformedXml(String),
}

pub fn canonicalize_exclusive(xml: &str) -> Result<String, C14nError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().check_comments = true;
    let mut out = String::new();
    let mut declared: Vec<BTreeMap<String, String>> = Vec::new();
    let mut rendered: Vec<BTreeMap<String, String>> = Vec::new();
    let mut depth = 0usize;

    loop {
        match reader
            .read_event()
            .map_err(|e| C14nError::MalformedXml(e.to_string()))?
        {
            Event::Start(tag) => {
                out.push_str(&render_start_tag(&tag, &mut declared, &mut rendered)?);
                depth += 1;
            }
            Event::Empty(tag) => {
                out.push_str(&render_start_tag(&tag, &mut declared, &mut rendered)?);
                out.push_str("</");
                out.push_str(&name_of(&tag)?);
                out.push('>');
                declared.pop();
                rendered.pop();
            }
            Event::End(tag) => {
                out.push_str("</");
                out.push_str(&utf8(tag.name().as_ref())?);
                out.push('>');
                depth = depth.saturating_sub(1);
                declared.pop();
                rendered.pop();
            }
            Event::Text(text) if depth > 0 => {
                let decoded = text
                    .xml10_content()
                    .map_err(|e| C14nError::MalformedXml(e.to_string()))?;
                out.push_str(&escape_text(&decoded));
            }
            Event::GeneralRef(reference) if depth > 0 => {
                let resolved = reference
                    .resolve_char_ref()
                    .map_err(|e| C14nError::MalformedXml(e.to_string()))?;
                let character = match resolved {
                    Some(character) => ensure_legal_xml_char(character)?,
                    None => {
                        let name = reference
                            .decode()
                            .map_err(|e| C14nError::MalformedXml(e.to_string()))?;
                        resolve_reference(&name, &name)?
                    }
                };
                out.push_str(&escape_text(&character.to_string()));
            }
            Event::CData(data) if depth > 0 => {
                let decoded = data
                    .xml10_content()
                    .map_err(|e| C14nError::MalformedXml(e.to_string()))?;
                out.push_str(&escape_text(&decoded));
            }
            Event::PI(instruction) => {
                out.push_str("<?");
                out.push_str(&utf8(instruction.as_ref())?);
                out.push_str("?>");
            }
            Event::Text(_) | Event::GeneralRef(_) | Event::CData(_) => {}
            Event::Comment(_) | Event::Decl(_) | Event::DocType(_) => {}
            Event::Eof => break,
        }
    }

    Ok(out)
}

fn render_start_tag(
    tag: &BytesStart<'_>,
    declared: &mut Vec<BTreeMap<String, String>>,
    rendered: &mut Vec<BTreeMap<String, String>>,
) -> Result<String, C14nError> {
    let name = name_of(tag)?;
    let mut declarations = BTreeMap::new();
    let mut attributes = Vec::new();

    for attribute in tag.attributes() {
        let attribute = attribute.map_err(|e| C14nError::MalformedXml(e.to_string()))?;
        let key = utf8(attribute.key.as_ref())?;
        let value = normalise_attribute_value(&utf8(attribute.value.as_ref())?)?;

        match key.strip_prefix("xmlns") {
            Some("") => {
                declarations.insert(String::new(), value);
            }
            Some(suffix) => match suffix.strip_prefix(':') {
                Some(prefix) => {
                    declarations.insert(prefix.to_owned(), value);
                }
                None => attributes.push((key, value)),
            },
            None => attributes.push((key, value)),
        }
    }

    declared.push(declarations);

    let mut utilized = BTreeSet::new();
    utilized.insert(prefix_of(&name).to_owned());
    for (key, _) in &attributes {
        let prefix = prefix_of(key);
        if !prefix.is_empty() && prefix != "xml" {
            utilized.insert(prefix.to_owned());
        }
    }

    let mut emitted = BTreeMap::new();
    let mut output = String::from("<");
    output.push_str(&name);

    for prefix in utilized {
        let Some(uri) = lookup(declared, &prefix) else {
            continue;
        };
        if lookup(rendered, &prefix).as_deref() == Some(uri.as_str()) {
            continue;
        }
        output.push_str(" xmlns");
        if !prefix.is_empty() {
            output.push(':');
            output.push_str(&prefix);
        }
        output.push_str("=\"");
        output.push_str(&escape_attribute_value(&uri));
        output.push('"');
        emitted.insert(prefix, uri);
    }

    rendered.push(emitted);

    let mut sortable: Vec<(String, String, String, String)> = attributes
        .into_iter()
        .map(|(key, value)| {
            let prefix = prefix_of(&key);
            let uri = if prefix.is_empty() {
                String::new()
            } else {
                lookup(declared, prefix).unwrap_or_default()
            };
            let local = local_of(&key).to_owned();
            (uri, local, key, value)
        })
        .collect();
    sortable.sort_by(|left, right| (&left.0, &left.1).cmp(&(&right.0, &right.1)));

    for (_, _, key, value) in sortable {
        output.push(' ');
        output.push_str(&key);
        output.push_str("=\"");
        output.push_str(&escape_attribute_value(&value));
        output.push('"');
    }

    output.push('>');
    Ok(output)
}

fn normalise_attribute_value(raw: &str) -> Result<String, C14nError> {
    let whitespace_normalised: String = raw
        .chars()
        .map(|character| match character {
            '\t' | '\n' | '\r' => ' ',
            other => other,
        })
        .collect();

    expand_references(&whitespace_normalised)
}

fn expand_references(value: &str) -> Result<String, C14nError> {
    let mut expanded = String::with_capacity(value.len());
    let mut rest = value;

    while let Some(start) = rest.find('&') {
        expanded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find(';') else {
            return Err(C14nError::MalformedXml(format!(
                "unterminated entity reference in attribute value: {value}"
            )));
        };
        expanded.push(resolve_reference(&after[..end], value)?);
        rest = &after[end + 1..];
    }

    expanded.push_str(rest);
    Ok(expanded)
}

fn resolve_reference(name: &str, context: &str) -> Result<char, C14nError> {
    let malformed =
        || C14nError::MalformedXml(format!("bad entity reference `&{name};` in {context}"));

    match name {
        "amp" => Ok('&'),
        "lt" => Ok('<'),
        "gt" => Ok('>'),
        "quot" => Ok('"'),
        "apos" => Ok('\''),
        _ => {
            let digits = name.strip_prefix('#').ok_or_else(malformed)?;
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hex) => u32::from_str_radix(hex, 16).map_err(|_| malformed())?,
                None => digits.parse::<u32>().map_err(|_| malformed())?,
            };
            let character = char::from_u32(code).ok_or_else(malformed)?;
            ensure_legal_xml_char(character)
        }
    }
}

fn ensure_legal_xml_char(character: char) -> Result<char, C14nError> {
    let code = character as u32;
    let legal = matches!(code, 0x9 | 0xA | 0xD)
        || (0x20..=0xD7FF).contains(&code)
        || (0xE000..=0xFFFD).contains(&code)
        || (0x10000..=0x10FFFF).contains(&code);

    if legal {
        Ok(character)
    } else {
        Err(C14nError::MalformedXml(format!(
            "character reference U+{code:04X} is outside the xml 1.0 character range"
        )))
    }
}

fn escape_attribute_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '"' => escaped.push_str("&quot;"),
            '\t' => escaped.push_str("&#x9;"),
            '\n' => escaped.push_str("&#xA;"),
            '\r' => escaped.push_str("&#xD;"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\r' => escaped.push_str("&#xD;"),
            other => escaped.push(other),
        }
    }
    escaped
}

fn prefix_of(name: &str) -> &str {
    match name.split_once(':') {
        Some((prefix, _)) => prefix,
        None => "",
    }
}

fn local_of(name: &str) -> &str {
    match name.split_once(':') {
        Some((_, local)) => local,
        None => name,
    }
}

const XML_NAMESPACE_URI: &str = "http://www.w3.org/XML/1998/namespace";

fn lookup(scopes: &[BTreeMap<String, String>], prefix: &str) -> Option<String> {
    if prefix == "xml" {
        return Some(XML_NAMESPACE_URI.to_owned());
    }
    scopes
        .iter()
        .rev()
        .find_map(|scope| scope.get(prefix))
        .cloned()
}

fn utf8(bytes: &[u8]) -> Result<String, C14nError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|e| C14nError::MalformedXml(e.to_string()))
}

fn name_of(tag: &BytesStart<'_>) -> Result<String, C14nError> {
    std::str::from_utf8(tag.name().as_ref())
        .map(str::to_owned)
        .map_err(|e| C14nError::MalformedXml(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::canonicalize_exclusive;

    #[test]
    fn plain_element_is_unchanged() {
        assert_eq!(
            canonicalize_exclusive("<a>t</a>").expect("canonicalise"),
            "<a>t</a>"
        );
    }

    #[test]
    fn namespace_is_emitted_on_the_element_that_uses_it_not_where_it_is_declared() {
        assert_eq!(
            canonicalize_exclusive(r#"<a xmlns:x="urn:x"><x:b>t</x:b></a>"#).expect("canonicalise"),
            r#"<a><x:b xmlns:x="urn:x">t</x:b></a>"#
        );
    }

    #[test]
    fn attributes_are_sorted_by_name() {
        assert_eq!(
            canonicalize_exclusive(r#"<a z="2" a="1" m="3">t</a>"#).expect("canonicalise"),
            r#"<a a="1" m="3" z="2">t</a>"#
        );
    }

    #[test]
    fn attributes_are_sorted_by_namespace_uri_before_local_name() {
        assert_eq!(
            canonicalize_exclusive(r#"<a xmlns:b="urn:z" xmlns:z="urn:a" b:p="1" z:q="2">t</a>"#)
                .expect("canonicalise"),
            r#"<a xmlns:b="urn:z" xmlns:z="urn:a" z:q="2" b:p="1">t</a>"#
        );
    }

    #[test]
    fn unprefixed_attributes_sort_before_prefixed_ones() {
        assert_eq!(
            canonicalize_exclusive(r#"<a xmlns:x="urn:x" x:p="1" q="2">t</a>"#)
                .expect("canonicalise"),
            r#"<a xmlns:x="urn:x" q="2" x:p="1">t</a>"#
        );
    }

    #[test]
    fn a_literal_tab_in_an_attribute_is_normalised_to_a_space() {
        assert_eq!(
            canonicalize_exclusive("<a v=\"x\ty\">t</a>").expect("canonicalise"),
            r#"<a v="x y">t</a>"#
        );
    }

    #[test]
    fn a_tab_written_as_a_character_reference_survives_as_one() {
        assert_eq!(
            canonicalize_exclusive(r#"<a v="x&#x9;y">t</a>"#).expect("canonicalise"),
            r#"<a v="x&#x9;y">t</a>"#
        );
    }

    #[test]
    fn an_ampersand_reference_in_an_attribute_becomes_its_named_entity() {
        assert_eq!(
            canonicalize_exclusive(r#"<a v="a &#38; b">t</a>"#).expect("canonicalise"),
            r#"<a v="a &amp; b">t</a>"#
        );
    }

    #[test]
    fn a_greater_than_in_an_attribute_is_left_alone() {
        assert_eq!(
            canonicalize_exclusive(r#"<a v="a > b">t</a>"#).expect("canonicalise"),
            r#"<a v="a > b">t</a>"#
        );
    }

    #[test]
    fn an_unused_namespace_declaration_is_dropped() {
        assert_eq!(
            canonicalize_exclusive(r#"<a xmlns:x="urn:x" xmlns:unused="urn:u"><x:b>t</x:b></a>"#)
                .expect("canonicalise"),
            r#"<a><x:b xmlns:x="urn:x">t</x:b></a>"#
        );
    }

    #[test]
    fn a_named_entity_in_text_is_expanded_then_re_escaped() {
        assert_eq!(
            canonicalize_exclusive("<a>a &amp; b</a>").expect("canonicalise"),
            "<a>a &amp; b</a>"
        );
    }

    #[test]
    fn every_predefined_entity_survives_a_round_trip() {
        assert_eq!(
            canonicalize_exclusive("<a>&lt;&gt;&amp;&quot;&apos;</a>").expect("canonicalise"),
            "<a>&lt;&gt;&amp;\"'</a>"
        );
    }

    #[test]
    fn a_character_reference_outside_the_xml_1_0_range_is_refused() {
        assert!(canonicalize_exclusive("<a>&#x1;</a>").is_err());
    }

    #[test]
    fn a_character_reference_outside_the_range_is_refused_in_attributes_too() {
        assert!(canonicalize_exclusive(r#"<a v="&#x1;">t</a>"#).is_err());
    }

    #[test]
    fn a_tab_reference_is_legal_and_survives() {
        assert_eq!(
            canonicalize_exclusive("<a>&#x9;</a>").expect("canonicalise"),
            "<a>\t</a>"
        );
    }

    #[test]
    fn a_comment_carrying_a_double_hyphen_is_refused() {
        assert!(canonicalize_exclusive("<a>t<!-- a -- b --></a>").is_err());
    }

    #[test]
    fn an_unknown_named_entity_is_refused() {
        assert!(canonicalize_exclusive("<a>&nbsp;</a>").is_err());
    }

    #[test]
    fn whitespace_outside_the_document_element_is_discarded() {
        assert_eq!(
            canonicalize_exclusive("  <a>t</a>  ").expect("canonicalise"),
            "<a>t</a>"
        );
    }

    #[test]
    fn cdata_line_endings_are_normalised() {
        assert_eq!(
            canonicalize_exclusive("<a><![CDATA[x\r\ny]]></a>").expect("canonicalise"),
            "<a>x\ny</a>"
        );
    }

    #[test]
    fn the_xml_prefix_sorts_by_its_implicit_namespace_uri() {
        assert_eq!(
            canonicalize_exclusive(r#"<a xmlns:p="http://a" p:x="1" xml:lang="en">t</a>"#)
                .expect("canonicalise"),
            r#"<a xmlns:p="http://a" p:x="1" xml:lang="en">t</a>"#
        );
    }

    #[test]
    fn processing_instructions_are_preserved() {
        assert_eq!(
            canonicalize_exclusive("<a>t<?target data?></a>").expect("canonicalise"),
            "<a>t<?target data?></a>"
        );
    }

    #[test]
    fn comments_are_removed() {
        assert_eq!(
            canonicalize_exclusive("<a>t<!--c--></a>").expect("canonicalise"),
            "<a>t</a>"
        );
    }

    #[test]
    fn a_default_namespace_stays_on_the_element_that_declares_it() {
        assert_eq!(
            canonicalize_exclusive(r#"<a xmlns="urn:d"><b>t</b></a>"#).expect("canonicalise"),
            r#"<a xmlns="urn:d"><b>t</b></a>"#
        );
    }

    #[test]
    fn a_literal_greater_than_in_text_is_escaped() {
        assert_eq!(
            canonicalize_exclusive("<a>a > b</a>").expect("canonicalise"),
            "<a>a &gt; b</a>"
        );
    }

    #[test]
    fn a_numeric_character_reference_becomes_its_named_entity() {
        assert_eq!(
            canonicalize_exclusive("<a>a &#60; b</a>").expect("canonicalise"),
            "<a>a &lt; b</a>"
        );
    }

    #[test]
    fn a_carriage_return_in_text_stays_a_numeric_reference() {
        assert_eq!(
            canonicalize_exclusive("<a>line1&#xD;line2</a>").expect("canonicalise"),
            "<a>line1&#xD;line2</a>"
        );
    }

    #[test]
    fn empty_element_is_expanded_to_a_start_and_end_tag() {
        assert_eq!(
            canonicalize_exclusive("<a><b/></a>").expect("canonicalise"),
            "<a><b></b></a>"
        );
    }
}
