use std::collections::BTreeMap;

use ferriskey_authz::tenant::FERRISKEY_NAMESPACE;
use serde_json::{Map, Value};

pub(crate) const CONTEXT_EXTENSION: &str = "ContextExtension";

pub(crate) const ENTITY_EXTENSIONS: [(&str, &str); 6] = [
    ("UserExtension", "User"),
    ("AgentExtension", "Agent"),
    ("ClientExtension", "Client"),
    ("ToolExtension", "Tool"),
    ("ResourceExtension", "Resource"),
    ("PromptExtension", "Prompt"),
];

const PRIMITIVES: [&str; 7] = [
    "String", "Long", "Bool", "ipaddr", "decimal", "datetime", "duration",
];

const MAX_DEPTH: usize = 32;

fn qualify(value: &mut Value, namespace: &str) {
    match value {
        Value::Object(object) => {
            let is_name = object.get("type").and_then(Value::as_str) == Some("EntityOrCommon");
            if is_name
                && let Some(Value::String(name)) = object.get_mut("name")
                && !name.contains("::")
                && !PRIMITIVES.contains(&name.as_str())
            {
                *name = format!("{namespace}::{name}");
            }
            for child in object.values_mut() {
                qualify(child, namespace);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| qualify(item, namespace)),
        _ => {}
    }
}

fn is_record(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str) == Some("Record")
}

fn without_required(value: &Value) -> Value {
    let mut copy = value.clone();
    if let Some(object) = copy.as_object_mut() {
        object.remove("required");
    }
    copy
}

fn set_required(target: &mut Value, source: &Value) {
    if let Some(object) = target.as_object_mut() {
        match source.get("required") {
            Some(required) => {
                object.insert("required".to_string(), required.clone());
            }
            None => {
                object.remove("required");
            }
        }
    }
}

fn merge_attributes(
    base: &mut Map<String, Value>,
    extension: &Map<String, Value>,
    path: &str,
) -> Result<(), String> {
    for (key, added) in extension {
        let here = format!("{path}.{key}");
        let Some(existing) = base.get_mut(key) else {
            base.insert(key.clone(), added.clone());
            continue;
        };
        if is_record(existing) && is_record(added) {
            let added_attributes = added
                .get("attributes")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            if let Some(Value::Object(inner)) = existing.get_mut("attributes") {
                merge_attributes(inner, &added_attributes, &here)?;
            }
            set_required(existing, added);
        } else if without_required(existing) == without_required(added) {
            set_required(existing, added);
        } else {
            return Err(format!(
                "{here} is a built-in attribute: its type cannot be changed, only made optional or required"
            ));
        }
    }
    Ok(())
}

fn record_attributes(name: &str, declared: &Value) -> Result<Map<String, Value>, String> {
    if !is_record(declared) {
        return Err(format!("{name} must be a record type"));
    }
    Ok(declared
        .get("attributes")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

pub(crate) fn extend_builtin(
    builtin: &mut Value,
    realm: &Value,
    namespace: &str,
) -> Result<(), String> {
    let declared = realm
        .get(namespace)
        .and_then(|ns| ns.get("commonTypes"))
        .and_then(Value::as_object);
    let Some(declared) = declared else {
        return Ok(());
    };
    let root = builtin
        .get_mut(FERRISKEY_NAMESPACE)
        .ok_or_else(|| "the built-in schema has no FerrisKey namespace".to_string())?;

    if let Some(extension) = declared.get(CONTEXT_EXTENSION) {
        let mut attributes = record_attributes(CONTEXT_EXTENSION, extension)?;
        attributes.values_mut().for_each(|a| qualify(a, namespace));
        let Some(Value::Object(base)) = root.pointer_mut("/commonTypes/AgentContext/attributes")
        else {
            return Err("the built-in AgentContext is missing".to_string());
        };
        merge_attributes(base, &attributes, "context")?;
    }

    for (extension_name, entity) in ENTITY_EXTENSIONS {
        let Some(extension) = declared.get(extension_name) else {
            continue;
        };
        let mut attributes = record_attributes(extension_name, extension)?;
        attributes.values_mut().for_each(|a| qualify(a, namespace));
        let Some(Value::Object(base)) =
            root.pointer_mut(&format!("/entityTypes/{entity}/shape/attributes"))
        else {
            return Err(format!("the built-in {entity} entity is missing"));
        };
        merge_attributes(base, &attributes, &entity.to_ascii_lowercase())?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
enum Shape {
    Record(BTreeMap<String, Shape>),
    Set(Box<Shape>),
    Leaf,
}

impl Shape {
    fn prune(&self, value: Value) -> Value {
        match (self, value) {
            (Self::Record(attributes), Value::Object(object)) => Value::Object(
                object
                    .into_iter()
                    .filter_map(|(key, value)| {
                        attributes.get(&key).map(|shape| (key, shape.prune(value)))
                    })
                    .collect(),
            ),
            (Self::Set(element), Value::Array(items)) => {
                Value::Array(items.into_iter().map(|item| element.prune(item)).collect())
            }
            (_, value) => value,
        }
    }
}

fn resolve(
    declared: &Value,
    namespaces: &Map<String, Value>,
    current: &str,
    depth: usize,
) -> Shape {
    if depth > MAX_DEPTH {
        return Shape::Leaf;
    }
    match declared.get("type").and_then(Value::as_str) {
        Some("Record") => {
            let attributes = declared
                .get("attributes")
                .and_then(Value::as_object)
                .map(|attributes| {
                    attributes
                        .iter()
                        .map(|(key, attribute)| {
                            (
                                key.clone(),
                                resolve(attribute, namespaces, current, depth + 1),
                            )
                        })
                        .collect()
                })
                .unwrap_or_default();
            Shape::Record(attributes)
        }
        Some("Set") => declared.get("element").map_or(Shape::Leaf, |element| {
            Shape::Set(Box::new(resolve(element, namespaces, current, depth + 1)))
        }),
        Some("EntityOrCommon") => {
            let Some(name) = declared.get("name").and_then(Value::as_str) else {
                return Shape::Leaf;
            };
            let (namespace, leaf) = name.rsplit_once("::").unwrap_or((current, name));
            namespaces
                .get(namespace)
                .and_then(|ns| ns.pointer(&format!("/commonTypes/{leaf}")))
                .map_or(Shape::Leaf, |common| {
                    resolve(common, namespaces, namespace, depth + 1)
                })
        }
        _ => Shape::Leaf,
    }
}

#[derive(Debug, Default)]
pub(crate) struct Shapes {
    entities: BTreeMap<String, Shape>,
    contexts: BTreeMap<String, Shape>,
}

impl Shapes {
    pub(crate) fn from_json(merged: &Value) -> Self {
        let mut shapes = Self::default();
        let Some(namespaces) = merged.as_object() else {
            return shapes;
        };
        let Some(root) = namespaces.get(FERRISKEY_NAMESPACE) else {
            return shapes;
        };
        if let Some(Value::Object(types)) = root.get("entityTypes") {
            for (name, entity) in types {
                if let Some(shape) = entity.get("shape") {
                    shapes.entities.insert(
                        format!("{FERRISKEY_NAMESPACE}::{name}"),
                        resolve(shape, namespaces, FERRISKEY_NAMESPACE, 0),
                    );
                }
            }
        }
        if let Some(Value::Object(actions)) = root.get("actions") {
            for (name, action) in actions {
                if let Some(context) = action.pointer("/appliesTo/context") {
                    let declared = match context.get("type").and_then(Value::as_str) {
                        Some("Record" | "EntityOrCommon") => context.clone(),
                        Some(common) => serde_json::json!({
                            "type": "EntityOrCommon",
                            "name": common,
                        }),
                        None => continue,
                    };
                    shapes.contexts.insert(
                        name.clone(),
                        resolve(&declared, namespaces, FERRISKEY_NAMESPACE, 0),
                    );
                }
            }
        }
        shapes
    }

    pub(crate) fn prune_entity(
        &self,
        entity_type: &str,
        attributes: Map<String, Value>,
    ) -> Map<String, Value> {
        match self.entities.get(entity_type) {
            Some(shape) => into_object(shape.prune(Value::Object(attributes))),
            None => attributes,
        }
    }

    pub(crate) fn prune_context(
        &self,
        action: &str,
        context: Map<String, Value>,
    ) -> Map<String, Value> {
        match self.contexts.get(action) {
            Some(shape) => into_object(shape.prune(Value::Object(context))),
            None => context,
        }
    }
}

fn into_object(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(object) => object,
        _ => Map::new(),
    }
}

pub(crate) fn sanitize(value: Value) -> Option<Value> {
    match value {
        Value::Null => None,
        Value::Number(number) => (number.is_i64()).then_some(Value::Number(number)),
        Value::Array(items) => Some(Value::Array(
            items.into_iter().filter_map(sanitize).collect(),
        )),
        Value::Object(object) => {
            if object
                .keys()
                .any(|key| key == "__entity" || key == "__extn")
            {
                return None;
            }
            Some(Value::Object(sanitize_object(object)))
        }
        other => Some(other),
    }
}

pub(crate) fn sanitize_object(object: Map<String, Value>) -> Map<String, Value> {
    object
        .into_iter()
        .filter_map(|(key, value)| sanitize(value).map(|value| (key, value)))
        .collect()
}
