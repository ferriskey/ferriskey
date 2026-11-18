use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use ferriskey_authz::tenant::FERRISKEY_NAMESPACE;
use serde_json::{Map, Value};

pub(crate) const CONTEXT_EXTENSION: &str = "ContextExtension";

const SERVER_BUILT: [&str; 2] = ["User", "Client"];

const CEDAR_PREFIX: &str = "__cedar::";

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

fn qualify(value: &mut Value, namespace: &str, local: &BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            let is_name = object.get("type").and_then(Value::as_str) == Some("EntityOrCommon");
            if is_name
                && let Some(Value::String(name)) = object.get_mut("name")
                && !name.contains("::")
                && (local.contains(name.as_str()) || !PRIMITIVES.contains(&name.as_str()))
            {
                *name = format!("{namespace}::{name}");
            }
            for child in object.values_mut() {
                qualify(child, namespace, local);
            }
        }
        Value::Array(items) => items
            .iter_mut()
            .for_each(|item| qualify(item, namespace, local)),
        _ => {}
    }
}

fn local_names(realm_namespace: &Value) -> BTreeSet<String> {
    ["commonTypes", "entityTypes"]
        .iter()
        .filter_map(|kind| realm_namespace.get(kind).and_then(Value::as_object))
        .flat_map(|declared| declared.keys().cloned())
        .collect()
}

fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let primitive = (object.get("type").and_then(Value::as_str) == Some("EntityOrCommon"))
                .then(|| object.get("name").and_then(Value::as_str))
                .flatten()
                .and_then(|name| name.strip_prefix(CEDAR_PREFIX))
                .filter(|name| PRIMITIVES.contains(name));
            Value::Object(
                object
                    .iter()
                    .map(|(key, child)| match (key.as_str(), primitive) {
                        ("name", Some(name)) => (key.clone(), Value::String(name.to_string())),
                        _ => (key.clone(), canonical(child)),
                    })
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

fn is_required(value: &Value) -> bool {
    value
        .get("required")
        .and_then(Value::as_bool)
        .unwrap_or(true)
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

fn server_supplies(
    entity: Option<&str>,
    here: &str,
    key: &str,
    added: &Value,
) -> Result<(), String> {
    match entity {
        Some(entity) if is_required(added) => Err(format!(
            "{here} must be optional (declare it as {key}?): FerrisKey builds {entity} entities itself and never sets it"
        )),
        _ => Ok(()),
    }
}

fn merge_attributes(
    base: &mut Map<String, Value>,
    extension: &Map<String, Value>,
    path: &str,
    server_built: Option<&str>,
) -> Result<(), String> {
    for (key, added) in extension {
        let here = format!("{path}.{key}");
        let Some(existing) = base.get_mut(key) else {
            server_supplies(server_built, &here, key, added)?;
            base.insert(key.clone(), added.clone());
            continue;
        };
        if !is_required(existing) {
            server_supplies(server_built, &here, key, added)?;
        }
        if is_record(existing) && is_record(added) {
            let added_attributes = added
                .get("attributes")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            if let Some(Value::Object(inner)) = existing.get_mut("attributes") {
                merge_attributes(inner, &added_attributes, &here, server_built)?;
            }
            set_required(existing, added);
        } else if canonical(&without_required(existing)) == canonical(&without_required(added)) {
            set_required(existing, added);
        } else {
            return Err(format!(
                "{here} is a built-in attribute: its type cannot be changed, only made optional or required"
            ));
        }
    }
    Ok(())
}

fn record_attributes(
    name: &str,
    declared: &Value,
    common: &Map<String, Value>,
    namespace: &str,
) -> Result<Map<String, Value>, String> {
    let mut current = declared;
    for _ in 0..=MAX_DEPTH {
        if is_record(current) {
            return Ok(current
                .get("attributes")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default());
        }
        let alias = (current.get("type").and_then(Value::as_str) == Some("EntityOrCommon"))
            .then(|| current.get("name").and_then(Value::as_str))
            .flatten()
            .map(|target| {
                target
                    .strip_prefix(namespace)
                    .and_then(|rest| rest.strip_prefix("::"))
                    .unwrap_or(target)
            })
            .and_then(|target| common.get(target));
        match alias {
            Some(target) => current = target,
            None => break,
        }
    }
    Err(format!("{name} must be a record type"))
}

pub(crate) fn extend_builtin(
    builtin: &mut Value,
    realm: &Value,
    namespace: &str,
) -> Result<(), String> {
    let Some(realm_namespace) = realm.get(namespace) else {
        return Ok(());
    };
    let Some(declared) = realm_namespace
        .get("commonTypes")
        .and_then(Value::as_object)
    else {
        return Ok(());
    };
    let local = local_names(realm_namespace);
    let root = builtin
        .get_mut(FERRISKEY_NAMESPACE)
        .ok_or_else(|| "the built-in schema has no FerrisKey namespace".to_string())?;

    if let Some(extension) = declared.get(CONTEXT_EXTENSION) {
        let mut attributes = record_attributes(CONTEXT_EXTENSION, extension, declared, namespace)?;
        attributes
            .values_mut()
            .for_each(|a| qualify(a, namespace, &local));
        let Some(Value::Object(base)) = root.pointer_mut("/commonTypes/AgentContext/attributes")
        else {
            return Err("the built-in AgentContext is missing".to_string());
        };
        merge_attributes(base, &attributes, "context", None)?;
    }

    for (extension_name, entity) in ENTITY_EXTENSIONS {
        let Some(extension) = declared.get(extension_name) else {
            continue;
        };
        let mut attributes = record_attributes(extension_name, extension, declared, namespace)?;
        attributes
            .values_mut()
            .for_each(|a| qualify(a, namespace, &local));
        let Some(Value::Object(base)) =
            root.pointer_mut(&format!("/entityTypes/{entity}/shape/attributes"))
        else {
            return Err(format!("the built-in {entity} entity is missing"));
        };
        let server_built = SERVER_BUILT.contains(&entity).then_some(entity);
        merge_attributes(
            base,
            &attributes,
            &entity.to_ascii_lowercase(),
            server_built,
        )?;
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum Shape {
    Record(BTreeMap<String, Arc<Shape>>),
    Set(Arc<Shape>),
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

struct Resolved {
    shape: Arc<Shape>,
    height: usize,
}

struct Resolver<'a> {
    namespaces: &'a Map<String, Value>,
    common: HashMap<(String, String), (Arc<Shape>, usize)>,
    leaf: Arc<Shape>,
}

impl<'a> Resolver<'a> {
    fn new(namespaces: &'a Map<String, Value>) -> Self {
        Self {
            namespaces,
            common: HashMap::new(),
            leaf: Arc::new(Shape::Leaf),
        }
    }

    fn leaf(&self) -> Resolved {
        Resolved {
            shape: self.leaf.clone(),
            height: 0,
        }
    }

    fn resolve(
        &mut self,
        declared: &Value,
        current: &str,
        depth: usize,
    ) -> Result<Resolved, String> {
        if depth > MAX_DEPTH {
            return Err(too_deep());
        }
        match declared.get("type").and_then(Value::as_str) {
            Some("Record") => {
                let mut attributes = BTreeMap::new();
                let mut height = 0;
                if let Some(declared) = declared.get("attributes").and_then(Value::as_object) {
                    for (key, attribute) in declared {
                        let resolved = self.resolve(attribute, current, depth + 1)?;
                        height = height.max(resolved.height + 1);
                        attributes.insert(key.clone(), resolved.shape);
                    }
                }
                Ok(Resolved {
                    shape: Arc::new(Shape::Record(attributes)),
                    height,
                })
            }
            Some("Set") => match declared.get("element") {
                Some(element) => {
                    let resolved = self.resolve(element, current, depth + 1)?;
                    Ok(Resolved {
                        shape: Arc::new(Shape::Set(resolved.shape)),
                        height: resolved.height + 1,
                    })
                }
                None => Ok(self.leaf()),
            },
            Some("EntityOrCommon") => {
                let Some(name) = declared.get("name").and_then(Value::as_str) else {
                    return Ok(self.leaf());
                };
                let (namespace, leaf) = name.rsplit_once("::").unwrap_or((current, name));
                let key = (namespace.to_string(), leaf.to_string());
                if let Some((shape, height)) = self.common.get(&key) {
                    if depth + 1 + height > MAX_DEPTH {
                        return Err(too_deep());
                    }
                    return Ok(Resolved {
                        shape: shape.clone(),
                        height: height + 1,
                    });
                }
                let namespaces = self.namespaces;
                match namespaces
                    .get(namespace)
                    .and_then(|ns| ns.pointer(&format!("/commonTypes/{leaf}")))
                {
                    Some(common) => {
                        let resolved = self.resolve(common, namespace, depth + 1)?;
                        self.common
                            .insert(key, (resolved.shape.clone(), resolved.height));
                        Ok(Resolved {
                            shape: resolved.shape,
                            height: resolved.height + 1,
                        })
                    }
                    None => Ok(self.leaf()),
                }
            }
            _ => Ok(self.leaf()),
        }
    }
}

fn too_deep() -> String {
    format!("the schema nests types deeper than {MAX_DEPTH} levels")
}

#[derive(Debug, Default)]
pub(crate) struct Shapes {
    entities: BTreeMap<String, Arc<Shape>>,
    contexts: BTreeMap<String, Arc<Shape>>,
}

impl Shapes {
    pub(crate) fn from_json(merged: &Value) -> Result<Self, String> {
        let mut shapes = Self::default();
        let Some(namespaces) = merged.as_object() else {
            return Ok(shapes);
        };
        let Some(root) = namespaces.get(FERRISKEY_NAMESPACE) else {
            return Ok(shapes);
        };
        let mut resolver = Resolver::new(namespaces);
        if let Some(Value::Object(types)) = root.get("entityTypes") {
            for (name, entity) in types {
                if let Some(shape) = entity.get("shape") {
                    shapes.entities.insert(
                        format!("{FERRISKEY_NAMESPACE}::{name}"),
                        resolver.resolve(shape, FERRISKEY_NAMESPACE, 0)?.shape,
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
                        resolver.resolve(&declared, FERRISKEY_NAMESPACE, 0)?.shape,
                    );
                }
            }
        }
        Ok(shapes)
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

const ESCAPES: [&str; 3] = ["__entity", "__extn", "__expr"];

pub(crate) fn sanitize(value: Value) -> Option<Value> {
    match value {
        Value::Null => None,
        Value::Number(number) => (number.is_i64()).then_some(Value::Number(number)),
        Value::Array(items) => Some(Value::Array(
            items.into_iter().filter_map(sanitize).collect(),
        )),
        Value::Object(object) => {
            if object.keys().any(|key| ESCAPES.contains(&key.as_str())) {
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_common_type_named_twice_resolves_to_one_shared_shape() {
        let namespaces = json!({
            "App": {"commonTypes": {
                "Twice": {"type": "Record", "attributes": {
                    "a": {"type": "EntityOrCommon", "name": "Once"},
                    "b": {"type": "EntityOrCommon", "name": "Once"},
                }},
                "Once": {"type": "Record", "attributes": {
                    "leaf": {"type": "EntityOrCommon", "name": "String"},
                }},
            }}
        });
        let namespaces = namespaces.as_object().expect("an object");
        let mut resolver = Resolver::new(namespaces);
        let resolved = resolver
            .resolve(
                &json!({"type": "EntityOrCommon", "name": "Twice"}),
                "App",
                0,
            )
            .expect("resolves");
        let Shape::Record(attributes) = resolved.shape.as_ref() else {
            panic!("a record");
        };
        assert!(Arc::ptr_eq(&attributes["a"], &attributes["b"]));
        assert_eq!(resolved.height, 4);
    }

    #[test]
    fn a_shared_shape_still_counts_toward_the_depth_limit() {
        let mut types = Map::new();
        for i in 0..20 {
            let next = if i == 19 {
                "String".to_string()
            } else {
                format!("T{}", i + 1)
            };
            types.insert(
                format!("T{i}"),
                json!({"type": "Record", "attributes": {
                    "a": {"type": "EntityOrCommon", "name": next},
                }}),
            );
        }
        let namespaces = json!({"App": {"commonTypes": Value::Object(types)}});
        let namespaces = namespaces.as_object().expect("an object");
        let mut resolver = Resolver::new(namespaces);
        assert!(
            resolver
                .resolve(&json!({"type": "EntityOrCommon", "name": "T10"}), "App", 0)
                .is_ok()
        );
        let error = resolver
            .resolve(&json!({"type": "EntityOrCommon", "name": "T0"}), "App", 0)
            .err()
            .expect("too deep through the shared T10");
        assert!(error.contains("deeper than"), "{error}");
    }
}
