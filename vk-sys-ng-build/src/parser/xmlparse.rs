//! XML parsing and document definition, as well as general passes over the whole document.



// Event-based parsers are well and good, but it's easier to work with an actual document tree than having to manually juggle events.
// So just define a simple XML document structure. The Vulkan spec XML is a few MB, so even allocation overhead of this
// naive implementation should not be too bad. It's also only paid once when generating.

use std::{collections::HashMap, fs::File, io::BufReader, iter::Peekable, sync::LazyLock};

use xml::reader::Events;

use crate::{VKParseError, parser::PResult};

#[derive(Debug, Clone)]
pub(crate) enum XMLNode {
    Element(XMLElement),
    Text(String)
}

#[derive(Debug, Clone)]
pub(crate) struct XMLElement {
    pub(crate) name: String,
    pub(crate) children: Vec<XMLNode>,
    pub(crate) attributes: HashMap<String, String>
}

impl XMLElement {
    pub(crate) fn text(&self) -> String {
        let mut text = String::new();
        for c in &self.children {
            match c {
                XMLNode::Element(xmlelement) => text += &xmlelement.text(),
                XMLNode::Text(t) => text += t.as_str(),
            }
        }
        return text;
    }
}

impl XMLNode {
    
    /// Calls f on itself and all children recursively until Some is returned.
    pub(crate) fn visit<F, R>(&self, mut f: F) -> Option<R> where F: FnMut(&XMLNode) -> Option<R> {
        if let Some(r) = f(self) {
            return Some(r);
        }
        match self {
            XMLNode::Element(xmlelement) => {
                for c in &xmlelement.children {
                    if let Some(r) = f(c) {
                        return Some(r);
                    }
                }
            },
            XMLNode::Text(_) => {},
        }
        return None;
    }
    
}


impl From<std::io::Error> for VKParseError {
    fn from(value: std::io::Error) -> Self {
        VKParseError::IOError(value)
    }
}

impl From<xml::reader::Error> for VKParseError {
    fn from(value: xml::reader::Error) -> Self {
        VKParseError::XMLError(value)
    }
}

impl From<&xml::reader::Error> for VKParseError {
    fn from(value: &xml::reader::Error) -> Self {
        VKParseError::XMLError(value.clone())
    }
}


pub(crate) type VKReader = Peekable<Events<BufReader<File>>>;




fn parse_node(r: &mut VKReader) -> PResult<XMLNode> {
    let e = r.peek().unwrap().as_ref()?;
    match e {
        xml::reader::XmlEvent::StartElement { name: _, attributes: _, namespace: _ } => {
            let e = parse_element(r)?;
            return Ok(XMLNode::Element(e));
        },
        xml::reader::XmlEvent::Characters(s) => {
            let ret = Ok(XMLNode::Text(s.clone()));
            r.next();
            return ret;
        },
        _ => {
            return Err(VKParseError::UnexpectedXMLElement(format!("Invalid Event: {:#?}", e)));
        }
    }
}

fn parse_element(r: &mut VKReader) -> PResult<XMLElement> {
    match r.next().unwrap()? {
        xml::reader::XmlEvent::StartElement { name, attributes, namespace: _ } => {
            let tag = name.local_name;
            let mut children = vec![];
            while let Some(p) = r.peek() {
                let p = p.as_ref()?;
                match p {
                    xml::reader::XmlEvent::EndElement { name } => {
                        if name.local_name != tag {
                            return Err(VKParseError::UnexpectedXMLElement(name.local_name.clone()));
                        }
                        r.next();
                        break;
                    },
                    _ => {
                        children.push(parse_node(r)?);
                    }
                }
            }
            return Ok(XMLElement { name: tag, children, attributes: attributes.into_iter().map(|a| (a.name.local_name, a.value)).collect() });
        }
        _ => {
            unreachable!()
        }
    };
    
}

pub(crate) fn parse_xml(r: &mut VKReader) -> PResult<XMLElement> {
    match r.next().unwrap()? {
        xml::reader::XmlEvent::StartDocument { version: _, encoding: _, standalone: _ } => {},
        _ => {
            unreachable!()
        }
    };
    let e = parse_element(r)?;
    match r.next().unwrap()? {
        xml::reader::XmlEvent::EndDocument => {}
        _ => {
            unreachable!()
        }
    };
    return Ok(e);
}


/// Filters comment nodes from the document tree.
pub(crate) fn filter_comments(e: &mut XMLElement) {
    e.children.retain_mut(|e| match e {
        XMLNode::Element(e) => {
            filter_comments(e);
            e.name != "comment"
        },
        XMLNode::Text(_) => true,
    });
}


/// Header remapping of header names not matching header files
pub(crate) static HEADER_REMAPS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("vk_platform".to_string(), "vk_platform.h".to_string());
    return m;
});

/// For some reason all includes are correctly named after their header files, except vk_platform. Fix this in a forward-compatible way.
pub(crate) fn fix_headers(e: &mut XMLElement) {
    if let Some(r) = e.attributes.get_mut("requires") {
        for h in HEADER_REMAPS.iter() {
            if r.contains(h.0) {
                *r = r.replace(h.0, h.1);
            }
        }
    }
    for c in &mut e.children {
        if let XMLNode::Element(c) = c {
            fix_headers(c);
        }
    }
}

/// Removes whitespace-only text nodes outside of elements that contain C code (where whitespace is important)
pub fn remove_whitespace_only_nodes(e: &mut XMLElement) {
    e.children.retain_mut(|e| match e {
        XMLNode::Element(e) => {
            match e.name.as_str() {
                // exclude elements where C code is the content
                "type" => {
                    // funcpointers, structs and unions are more like cmd tags and need whitespace removed. The proto and param nodes in them are protected though.
                    if let Some(cat) = e.attributes.get("category") && (
                            cat == "funcpointer" ||
                            cat == "struct" ||
                            cat == "union") {
                        remove_whitespace_only_nodes(e);
                    }
                }
                "proto"|"param" => {},
                _ => {
                    remove_whitespace_only_nodes(e);
                }
            }
            true
        },
        XMLNode::Text(t) => t.trim().len() != 0,
    });
}
