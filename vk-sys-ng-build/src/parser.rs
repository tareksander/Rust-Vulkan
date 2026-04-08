use std::{collections::{HashMap, HashSet}, fs::File, io::BufReader, iter::Peekable, path::Path, sync::LazyLock};

use logos::Logos;
use xml::{EventReader, ParserConfig, reader::Events};

use crate::data::{CFunc, CPrimitive, CType, CTypeDef, CmdBufferLevel, CommandTask, EnumDeprecation, Externsync, InRenderpass, LimitType, ParamArrayLen, TypeDeprecation, VKCmd, VKCmdDefinition, VKEnum, VKEnumType, VKEnumVariant, VKPlatform, VKRegistry, VKStruct, VKStructMember, VKTypeDefinition, VKTypeDefinitionKind};

mod cparse;
use cparse::*;

mod xmlparse;
use xmlparse::*;


#[derive(Debug)]
pub enum VKParseError {
    /// An IO error has occurred while reading the spec file.
    IOError(std::io::Error),
    /// The spec file is malformed XML
    XMLError(xml::reader::Error),
    UnexpectedXMLElement(String),
}


static EXT_CONSTS: &'static str = "Extension Constants";


type PResult<T> = Result<T, VKParseError>;



pub fn parse_registry(path: &Path) -> Result<VKRegistry, VKParseError> {
    let mut reader: VKReader = EventReader::new_with_config(BufReader::new(File::open(path)?),
    ParserConfig::new()
        .coalesce_characters(true)
        .whitespace_to_characters(true)).into_iter().peekable();
    let mut doc = parse_xml(&mut reader).unwrap();
    filter_comments(&mut doc);
    fix_headers(&mut doc);
    remove_whitespace_only_nodes(&mut doc);
    return parse_registry_element(&doc);
}



fn parse_registry_element(e: &XMLElement) -> PResult<VKRegistry> {
    if e.name != "registry" {
        return Err(VKParseError::UnexpectedXMLElement(e.name.clone()));
    }
    let mut types = None;
    let mut platforms = None;
    let mut commands = None;
    let mut enums = HashMap::new();
    enums.insert(EXT_CONSTS.to_string(), VKEnum { ty: VKEnumType::Constants, bitwidth: 32, variants: vec![] });
    for c in &e.children {
        match c {
            XMLNode::Element(c) => {
                //println!("{}", c.name);
                match c.name.as_str() {
                    "platforms" => {
                        platforms = Some(parse_platforms(c)?);
                    },
                    "tags" => {
                        
                    },
                    "types" => {
                        types = Some(parse_types(c, &mut enums)?);
                    },
                    "enums" => {
                        parse_enums(&mut enums, c);
                    },
                    "commands" => {
                        commands = Some(parse_commands(c));
                    }
                    "feature" => {
                        add_required_enum_variants(c, &mut enums);
                    },
                    "extensions" => {
                        for c in &c.children {
                            match c {
                                // exclude provisional extensions
                                XMLNode::Element(c) => {
                                    if c.attributes.get("provisional") != Some(&"true".to_string()) &&
                                        c.attributes.get("supported") != Some(&"disabled".to_string()) {
                                            add_required_enum_variants(c, &mut enums);
                                    }
                                },
                                XMLNode::Text(_) => {},
                            }
                        }
                    },
                    "formats" => {
                        
                    },
                    "spirvextensions" => {
                        
                    },
                    "spirvcapabilities" => {
                        
                    }
                    _ => {}
                }
            },
            XMLNode::Text(s) => panic!("Text in registry tag: {}", s),
        }
    }
    return Ok(VKRegistry {
        platforms: platforms.unwrap(),
        types: types.unwrap(),
        commands: commands.unwrap(),
        enums
    });
}

fn add_required_enum_variants(ext_element: &XMLElement, enums: &mut HashMap<String, VKEnum>) {
    //println!("{:#?}", ext_element);
    let extnum: Option<u32> = if ext_element.name == "extension" {
        ext_element.attributes.get("number").map(|s| s.parse().unwrap())
    } else {
        None
    };
    for c in &ext_element.children {
        match c {
            XMLNode::Element(re) => {
                if re.name == "require" {
                    for c in &re.children {
                        match c {
                            XMLNode::Element(c) => {
                                // skip reference enums
                                if c.name == "enum" && c.attributes.keys().any(|s| s != "name" && s != "comment" && s != "api")  {
                                    let name = c.attributes.get("name").unwrap().clone();
                                    let extends = c.attributes.get("extends").cloned().unwrap_or_else(|| EXT_CONSTS.to_string());
                                    if let Some(alias) = c.attributes.get("alias") {
                                        enums.get_mut(&extends).unwrap().variants.push(VKEnumVariant {
                                            name,
                                            value: None,
                                            bitpos: None,
                                            deprecated: EnumDeprecation::None,
                                            ty: None,
                                            alias: Some(alias.clone()),
                                        });
                                        continue;
                                    }
                                    if let Some(v) = c.attributes.get("value") {
                                        enums.get_mut(&extends).unwrap().variants.push(VKEnumVariant {
                                            name,
                                            value: Some(v.clone()),
                                            bitpos: None,
                                            deprecated: EnumDeprecation::None,
                                            ty: c.attributes.get("type").map(|s| match parse_c_type(&lex_c(s.as_str())) {
                                                CType::Primitive(cprimitive) => cprimitive,
                                                t => panic!("Invalid enum type: {:#?}", t)
                                            }),
                                            alias: None
                                        });
                                        continue;
                                    }
                                    if let Some(bitpos) = c.attributes.get("bitpos") {
                                        enums.get_mut(&extends).unwrap().variants.push(VKEnumVariant {
                                            name,
                                            value: None,
                                            bitpos: Some(bitpos.parse().unwrap()),
                                            deprecated: EnumDeprecation::None,
                                            ty: None,
                                            alias: None
                                        });
                                        continue;
                                    }
                                    let extnum = extnum.or(c.attributes.get("extnumber").map(|s| s.parse().unwrap())).expect("enum extension in feature block without extension offset");
                                    const EXT_ENUM_BASE: u32 = 1000000000;
                                    const EXT_ENUM_RANGE: u32 = 1000;
                                    let offset: u32 = c.attributes.get("offset").unwrap().parse().unwrap();
                                    let mut value = (EXT_ENUM_BASE + (extnum - 1) * EXT_ENUM_RANGE + offset) as i64;
                                    if c.attributes.contains_key("dir") {
                                        value = - value;
                                    }
                                    enums.get_mut(&extends).unwrap().variants.push(VKEnumVariant {
                                        name,
                                        value: Some(value.to_string()),
                                        bitpos: None,
                                        deprecated: EnumDeprecation::None,
                                        ty: None,
                                        alias: None
                                    });
                                }
                            },
                            XMLNode::Text(_) => {},
                        }
                    }
                }
            },
            XMLNode::Text(_) => {},
        }
    }
}

fn parse_commands(cmds_element: &XMLElement) -> HashMap<String, VKCmd> {
    let mut commands = HashMap::new();
    
    for c in &cmds_element.children {
        match c {
            XMLNode::Element(c) =>  {
                let name = get_name(c).unwrap();
                if let Some(alias) = c.attributes.get("alias") {
                    commands.insert(name, VKCmd::Alias(alias.clone()));
                } else {
                    let content = parse_cmd(c);
                    commands.insert(name, VKCmd::Definition(VKCmdDefinition {
                        tasks: c.attributes.get("tasks").map(|s| s.split(",").map(|s| match s {
                            "action" => CommandTask::Action,
                            "indirection" => CommandTask::Indirect,
                            "state" => CommandTask::State,
                            "synchronization" => CommandTask::Sync,
                            s => panic!("Unknown task: {}", s)
                        }).fold(CommandTask::empty(), |a, b| a | b)).unwrap_or(CommandTask::empty()),
                        queues: c.attributes.get("queues").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
                        success_codes: c.attributes.get("successcodes").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
                        error_code: c.attributes.get("errorcodes").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
                        in_renderpass: c.attributes.get("renderpass").map(|s| match s.as_str() {
                            "inside" => InRenderpass::Yes,
                            "outside" => InRenderpass::No,
                            "both" => InRenderpass::Both,
                            s => panic!("Unknown renderpass attribute: {}", s)
                        }).unwrap_or(InRenderpass::Both),
                        level: c.attributes.get("cmdbufferlevel").map(|s| match (s.contains("primary"), s.contains("secondary")) {
                            (true, true) => CmdBufferLevel::Both,
                            (true, false) => CmdBufferLevel::Primary,
                            (false, true) => CmdBufferLevel::Secondary,
                            (false, false) => panic!("invalid command buffer level")
                        }).unwrap_or(CmdBufferLevel::Both),
                        signature: content,
                    }));
                }
            },
            XMLNode::Text(_) => panic!("Text in commands element"),
        }
    }
    
    
    return commands;
}


fn parse_enums(enums: &mut HashMap<String, VKEnum>, enums_element: &XMLElement) {
    
    let name = enums_element.attributes.get("name")
        .expect("enums element without name attribute. While this is technically valid, all enums currently in the spec have them.").clone();
    
    let bitwidth = enums_element.attributes.get("bitwidth").map(|s| s.parse().unwrap()).unwrap_or(32u16);
    let ty = match enums_element.attributes.get("type").expect("enums element without type attribute").as_str() {
        "bitmask" => {
            VKEnumType::Bitmask
        },
        "constants" => {
            VKEnumType::Constants
        },
        "enum" => {
            VKEnumType::Enum
        },
        
        s => panic!("unknown enum type: {}", s)
    };
    
    let mut variants = vec![];
    
    for c in &enums_element.children {
        match c {
            XMLNode::Element(c) => {
                if c.name == "unused" {
                    continue;
                }
                //println!("{:#?}", c);
                variants.push(VKEnumVariant {
                    name: c.attributes.get("name").unwrap().to_string(),
                    value: c.attributes.get("value").cloned(),
                    bitpos: c.attributes.get("bitpos").map(|s| s.parse().unwrap()),
                    deprecated: c.attributes.get("deprecated").map(|s| match s.as_str() {
                        "true" => EnumDeprecation::Deprecated,
                        "ignored" => EnumDeprecation::Ignored,
                        "aliased" => EnumDeprecation::Aliased,
                        s => panic!("unknown enum deprecation: {}", s)
                    }).unwrap_or(EnumDeprecation::None),
                    ty: c.attributes.get("type").map(|s| match parse_c_type(&lex_c(s.as_str())) {
                        CType::Primitive(p) => p,
                        t => panic!("Enums type is not a primitive: {:#?}", t)
                    }),
                    alias: c.attributes.get("alias").cloned(),
                });
            },
            XMLNode::Text(_) => panic!("Text in enums element"),
        }
    }
    
    
    enums.insert(name, VKEnum { ty, bitwidth, variants });
}

fn parse_platforms(platforms_element: &XMLElement) -> PResult<Vec<VKPlatform>> {
    let mut platforms = vec![];
    
    for c in &platforms_element.children {
        match c {
            XMLNode::Element(c) => {
                platforms.push(VKPlatform {
                    name: c.attributes["name"].clone(),
                    protect: c.attributes["protect"].clone(),
                });
            },
            _ => panic!("Text in platforms element")
        }
    }
    
    
    return Ok(platforms);
}


/// Some types to skip the parser can't handle (mostly ObjC stuff).
static SKIP_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| vec![
    "CAMetalLayer",
    "MTLDevice_id",
    "MTLCommandQueue_id",
    "MTLBuffer_id",
    "MTLTexture_id",
    "MTLSharedEvent_id",
].into_iter().collect());




impl TypeDeprecation {
    fn parse(d: Option<&String>) -> Self {
        if let Some(d) = d {
            match d.as_str() {
                "true" => TypeDeprecation::Legacy,
                "aliased" => TypeDeprecation::Aliased,
                _ => panic!("Unknown deprecation state: {}", d)
            }
        } else {
            TypeDeprecation::None
        }
    }
}



/// Parses a member/param element
fn parse_member_param(e: &mut XMLElement) -> VKStructMember {
    let mut name: Option<String> = None;
    e.children.retain(|c| match c {
        XMLNode::Element(e) => {
            if e.name == "name" {
                name = Some(e.text());
                false
            } else {
                true
            }
        },
        XMLNode::Text(_) => true,
    });
    let name = name.expect("param/member tag without name tag");
    let param_text = e.text();
    let param_tokens = lex_c(&param_text);
    let ty = parse_c_type(&param_tokens);
    //println!("{:#?}", e);
    return VKStructMember {
        name,
        ty,
        stride: e.attributes.get("stride").cloned(),
        len: e.attributes.get("len").map(|l| l.split(",").map(|s| match s.trim() {
            "null-terminated" => ParamArrayLen::NullTerminated,
            "1" => ParamArrayLen::One,
            s => {
                if s.chars().all(|c| c.is_ascii_alphanumeric()) {
                    ParamArrayLen::Named(s.to_string())
                } else {
                    ParamArrayLen::Custom(s.to_string())
                }
            }
        }).collect()).unwrap_or(vec![]),
        optional: e.attributes.get("optional").map(|s| s.split(",").map(|s| match s {
            "true" => true,
            "false" => false,
            _ => panic!("Unknown optional value: {}", s)
        }).collect()).unwrap_or(vec![]),
        selector: e.attributes.get("selector").cloned(),
        selection: e.attributes.get("selection").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
        externsync: e.attributes.get("externsync").map(|s| match s.as_str() {
            "true" => Externsync::Yes,
            "false" => Externsync::No,
            "maybe" => Externsync::Maybe,
            s => {
                println!("Warning: unknown externsync value, guessing maybe: {}", s);
                Externsync::Maybe
            }
        }).unwrap_or(Externsync::No),
        valid_structs: e.attributes.get("validstructs").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
        feature_link: e.attributes.get("featurelink").cloned(),
        limit_type: e.attributes.get("limittype").map(|l| l.split(",").map(|l| match l {
            "min" => LimitType::Min,
            "max" => LimitType::Max,
            "pot" => LimitType::Pot,
            "mul" => LimitType::Mul,
            "bits" => LimitType::Bits,
            "bitmask" => LimitType::Bitmask,
            "range" => LimitType::Range,
            "struct" => LimitType::Struct,
            "exact" => LimitType::Exact,
            "noauto" => LimitType::NoAuto,
            _ => panic!("Unknown limit type: {}", l)
        }).fold(LimitType::empty(), |a, b| a | b)),
        values: e.attributes.get("values").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
    };
}

/// Parses the contents of a command/funcpointer tag.
fn parse_cmd(cmd: &XMLElement) -> CFunc {
    //println!("{:#?}", cmd);
    let mut proto = match &cmd.children[0] {
        XMLNode::Element(e) => {
            e.clone()
        }
        _ => unreachable!()
    };
    proto.children.retain(|c| match c {
        XMLNode::Element(e) => e.name != "name",
        XMLNode::Text(_) => true,
    });
    let ret = proto.text();
    let ret_tokens = lex_c(&ret);
    let ret = parse_c_type(&ret_tokens);
    
    let mut params = vec![];
    
    for p in &cmd.children[1..] {
        let mut param = match p {
            XMLNode::Element(e) => {
                e.clone()
            }
            _ => unreachable!()
        };
        if param.name == "implicitexternsyncparams" {
            continue;
        }
        if param.name != "param" {
            panic!("Unknown tag in command/funcpointer definition: {}", param.name);
        }
        
        params.push(parse_member_param(&mut param));
    }
    return CFunc {
        ret,
        params,
    };
}

fn parse_struct(e: &XMLElement) -> VKStruct {
    let members = e.children.iter().filter_map(|e| match e {
        XMLNode::Element(e) => Some(parse_member_param(&mut e.clone())),
        XMLNode::Text(s) => panic!("Text in struct type element: {}", s),
    }).collect();
    return VKStruct {
        allow_duplicate: e.attributes.get("allowduplicate").map(|s| match s.as_str() {
            "true" => true,
            "false" => false,
            _ => panic!("Unknown allowduplicate value: {}", s)
        }).unwrap_or(false),
        required_limit_type: e.attributes.get("requiredlimittype").map(|s| match s.as_str() {
            "true" => true,
            "false" => false,
            _ => panic!("Unknown requiredlimittype value: {}", s)
        }).unwrap_or(false),
        returned_only: e.attributes.get("returnedonly").map(|s| match s.as_str() {
            "true" => true,
            "false" => false,
            _ => panic!("Unknown returnedonly value: {}", s)
        }).unwrap_or(false),
        struct_extends: e.attributes.get("structextends").map(|s| s.split(",").map(|s| s.to_string()).collect()).unwrap_or(vec![]),
        members,
    };
}

fn get_name(e: &XMLElement) -> Option<String> {
    if let Some(name) = e.attributes.get("name") {
        return Some(name.clone());
    } else {
        for c in &e.children {
            if let Some(name) = c.visit(|n| {
                if let XMLNode::Element(e) = n {
                    if e.name == "name" {
                        return Some(e.text());
                    }
                }
                return None;
            }) {
                return Some(name);
            }
        }
    }
    return None;
}
fn parse_types(e: &XMLElement, enums: &mut HashMap<String, VKEnum>) -> PResult<HashMap<String, VKTypeDefinition>> {
    
    fn unwrap_name(types_element: &XMLElement, name: Option<String>) -> String {
        if let Some(name) = name {
            return name;
        } else {
            panic!("Type definition without name: {:#?}", types_element);
        }
    }
    
    /// Handles type aliases. Returns true if the type isn't an alias.
    fn handle_alias(e: &XMLElement, types: &mut HashMap<String, VKTypeDefinition>) -> bool {
        if let (Some(name), Some(alias)) = (e.attributes.get("name"), e.attributes.get("alias")) {
            types.insert(name.clone(), VKTypeDefinition {
                requires: e.attributes.get("requires").cloned(),
                deprecated: TypeDeprecation::parse(e.attributes.get("deprecated")),
                kind: VKTypeDefinitionKind::Alias(alias.clone()) });
            return false;
        }
        return true;
    }
    
    let mut types = HashMap::new();
    for c in &e.children {
        match c {
            XMLNode::Element(c) => {
                //println!("{}", c.name);
                let requires = c.attributes.get("requires").cloned();
                let deprecated = TypeDeprecation::parse(c.attributes.get("deprecated"));
                match c.name.as_str() {
                    "type" => {
                        if let Some(cat) = c.attributes.get("category") {
                            match cat.as_str() {
                                "include" => {
                                    let name;
                                    if let Some(v) = HEADER_REMAPS.get(&c.attributes["name"]) {
                                        name = v
                                    } else {
                                        name = &c.attributes["name"]
                                    }
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    types.insert(name.clone(), VKTypeDefinition {
                                        requires,
                                        deprecated: TypeDeprecation::None,
                                        kind: VKTypeDefinitionKind::Include(c.text()),
                                    });
                                },
                                "basetype" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    //println!("basetype: {}", &name);
                                    //println!("{}", c.text());
                                    let ty = parse_c_type_def(&lex_c(c.text().as_str()));
                                    types.insert(name, VKTypeDefinition { requires, deprecated, kind: VKTypeDefinitionKind::BaseType(ty) });
                                },
                                "bitmask" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    if handle_alias(c, &mut types) {
                                        enums.insert(name, VKEnum { ty: VKEnumType::Bitmask, bitwidth: if c.text().contains("VkFlags64") {
                                            64
                                        } else {
                                            32
                                        }, variants: vec![] });
                                    }
                                },
                                "define" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    types.insert(name, VKTypeDefinition { requires, deprecated, kind: VKTypeDefinitionKind::Define(c.text()) });
                                },
                                "enum" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    if handle_alias(c, &mut types) {
                                        enums.insert(name, VKEnum { ty: VKEnumType::Enum, bitwidth: 32, variants: vec![] });
                                    }
                                },
                                "funcpointer" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    let ty = parse_cmd(c);
                                    types.insert(name, VKTypeDefinition { requires, deprecated, kind: VKTypeDefinitionKind::FunctionPointer(ty) });
                                },
                                "handle" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    if handle_alias(c, &mut types) {
                                        let dispatchable;
                                        let text = c.text();
                                        if text.contains("VK_DEFINE_HANDLE") {
                                            dispatchable = true;
                                        } else {
                                            if ! text.contains("VK_DEFINE_NON_DISPATCHABLE_HANDLE") {
                                                panic!("neither dispatchable nor non-dispatchable handle: {}", name);
                                            }
                                            dispatchable = false;
                                        }
                                        types.insert(name, VKTypeDefinition { requires, deprecated, kind: VKTypeDefinitionKind::Handle { dispatchable } });
                                    }
                                },
                                "struct" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    if handle_alias(c, &mut types) {
                                        types.insert(name, VKTypeDefinition { requires, deprecated, kind: VKTypeDefinitionKind::Struct(parse_struct(c)) });
                                    }
                                },
                                "union" => {
                                    let name = unwrap_name(c, get_name(c));
                                    if  SKIP_TYPES.contains(name.as_str()) {
                                        continue;
                                    }
                                    if handle_alias(c, &mut types) {
                                        types.insert(name, VKTypeDefinition { requires, deprecated, kind: VKTypeDefinitionKind::Struct(parse_struct(c)) });
                                    }
                                },
                                _ => panic!("Unknown type category: {}", cat)
                            }
                        }  else {
                            if let (Some(name), Some(requires)) = (c.attributes.get("name"), c.attributes.get("requires")) {
                                types.insert(name.clone(), VKTypeDefinition {
                                    requires: Some(requires.clone()),
                                    deprecated: TypeDeprecation::None,
                                    kind: VKTypeDefinitionKind::Dependency,
                                });
                            } else {
                                panic!("categoryless type without name and requires: {:#?}", c);
                            }
                        }
                    }
                    _ => {
                        panic!("Unexpected tag in types tag: {}", e.name);
                    }
                }
            },
            XMLNode::Text(s) => panic!("Text in types tag: {}", s),
        }
    }
    
    return Ok(types);
}









