use std::collections::HashMap;
use std::str::FromStr;

use cedar_policy::{
    EntityId, EntityTypeName, EntityUid, Policy as CedarPolicy, PolicyId, PolicySet, Schema,
    SchemaFragment, SlotId as CedarSlotId, Template, ValidationMode, Validator,
};
use ferriskey_authz::entities::EntityRef;
use ferriskey_authz::tenant::entities::{AuthzSchema, Policy, PolicyBody, PolicyTemplate};
use ferriskey_authz::tenant::names::SlotId;
use ferriskey_authz::tenant::ports::PolicyCompiler;
use ferriskey_domain::common::app_errors::PolicyIssue;
use uuid::Uuid;

use crate::builtin::builtin_fragment;
use crate::issues::{issue, issues_from, plain};

#[derive(Debug, Default)]
pub struct CedarPolicyCompiler;

impl CedarPolicyCompiler {
    pub fn new() -> Self {
        Self
    }
}

pub(crate) struct Origin<'a> {
    name: &'a str,
    source: &'a str,
}

pub(crate) fn cedar_id(id: Uuid) -> PolicyId {
    PolicyId::new(id.to_string())
}

pub(crate) fn compile_schema(schema: &AuthzSchema) -> Result<Schema, Vec<PolicyIssue>> {
    let (fragment, _warnings) = SchemaFragment::from_cedarschema_str(&schema.source)
        .map_err(|e| issues_from(None, &e, &schema.source))?;

    let foreign: Vec<PolicyIssue> = fragment
        .namespaces()
        .filter(|ns| {
            ns.as_ref().map(ToString::to_string).as_deref() != Some(schema.namespace.as_str())
        })
        .map(|ns| {
            let declared = ns.map_or_else(
                || "the empty namespace".to_string(),
                |n| format!("namespace {n}"),
            );
            plain(
                None,
                format!(
                    "declares types in {declared}; declare them in namespace {}",
                    schema.namespace
                ),
            )
        })
        .collect();
    if !foreign.is_empty() {
        return Err(foreign);
    }

    Schema::from_schema_fragments([builtin_fragment()?, fragment])
        .map_err(|e| issues_from(None, &e, &schema.source))
}

fn link_values(
    links: &std::collections::BTreeMap<SlotId, EntityRef>,
    owner: &str,
) -> Result<HashMap<CedarSlotId, EntityUid>, Vec<PolicyIssue>> {
    links
        .iter()
        .map(|(slot, entity)| {
            let ty = EntityTypeName::from_str(entity.ty.as_str())
                .map_err(|e| issues_from(Some(owner), &e, entity.ty.as_str()))?;
            let uid = EntityUid::from_type_name_and_id(ty, EntityId::new(&entity.id));
            let slot = match slot {
                SlotId::Principal => CedarSlotId::principal(),
                SlotId::Resource => CedarSlotId::resource(),
            };
            Ok((slot, uid))
        })
        .collect()
}

pub(crate) fn build_policy_set<'a>(
    policies: impl IntoIterator<Item = &'a Policy>,
    templates: &'a [PolicyTemplate],
) -> Result<(PolicySet, HashMap<PolicyId, Origin<'a>>), Vec<PolicyIssue>> {
    let mut set = PolicySet::new();
    let mut origins = HashMap::with_capacity(templates.len());
    let mut issues = Vec::new();

    for template in templates {
        let name = template.name.as_str();
        let id = cedar_id(template.id);
        match Template::parse(Some(id.clone()), &template.source) {
            Ok(parsed) => match set.add_template(parsed) {
                Ok(()) => {
                    origins.insert(
                        id,
                        Origin {
                            name,
                            source: &template.source,
                        },
                    );
                }
                Err(e) => issues.extend(issues_from(Some(name), &e, &template.source)),
            },
            Err(e) => issues.extend(issues_from(Some(name), &e, &template.source)),
        }
    }

    for policy in policies {
        let name = policy.name.as_str();
        let id = cedar_id(policy.id);
        match &policy.body {
            PolicyBody::Static { source } => {
                let added = CedarPolicy::parse(Some(id.clone()), source)
                    .map_err(|e| issues_from(Some(name), &e, source))
                    .and_then(|parsed| {
                        set.add(parsed)
                            .map_err(|e| issues_from(Some(name), &e, source))
                    });
                match added {
                    Ok(()) => {
                        origins.insert(id, Origin { name, source });
                    }
                    Err(found) => issues.extend(found),
                }
            }
            PolicyBody::Linked { template_id, links } => {
                let template_id = cedar_id(*template_id);
                let Some(source) = origins.get(&template_id).map(|o| o.source) else {
                    issues.push(plain(
                        Some(name),
                        "links a template that does not compile".to_string(),
                    ));
                    continue;
                };
                let linked = link_values(links, name).and_then(|values| {
                    set.link(template_id, id.clone(), values)
                        .map_err(|e| issues_from(Some(name), &e, source))
                });
                match linked {
                    Ok(()) => {
                        origins.insert(id, Origin { name, source });
                    }
                    Err(found) => issues.extend(found),
                }
            }
        }
    }

    if issues.is_empty() {
        Ok((set, origins))
    } else {
        Err(issues)
    }
}

fn validate_against(
    schema: Schema,
    set: &PolicySet,
    origins: &HashMap<PolicyId, Origin<'_>>,
) -> Result<(), Vec<PolicyIssue>> {
    let result = Validator::new(schema).validate(set, ValidationMode::Strict);
    let issues: Vec<PolicyIssue> = result
        .validation_errors()
        .map(|error| match origins.get(error.policy_id()) {
            Some(origin) => issue(Some(origin.name), error, origin.source),
            None => issue(None, error, ""),
        })
        .collect();

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

impl PolicyCompiler for CedarPolicyCompiler {
    fn validate_schema(&self, schema: &AuthzSchema) -> Result<(), Vec<PolicyIssue>> {
        compile_schema(schema).map(|_| ())
    }

    fn validate_policies(
        &self,
        schema: Option<&AuthzSchema>,
        policies: &[Policy],
        templates: &[PolicyTemplate],
    ) -> Result<(), Vec<PolicyIssue>> {
        let (set, origins) = build_policy_set(policies, templates)?;
        match schema {
            Some(schema) => validate_against(compile_schema(schema)?, &set, &origins),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::Utc;
    use ferriskey_authz::entities::{EntityType, PolicyOrigin};
    use ferriskey_authz::tenant::names::{Namespace, PolicyName};

    use super::*;

    const APP_SCHEMA: &str = r#"namespace App {
  entity Document = {
    owner: FerrisKey::User,
  };

  action "document:read" appliesTo {
    principal: [FerrisKey::User],
    resource: [Document],
  };
}
"#;

    fn schema(source: &str) -> AuthzSchema {
        AuthzSchema {
            id: Uuid::new_v4(),
            namespace: Namespace::default(),
            source: source.to_string(),
            updated_at: Utc::now(),
        }
    }

    fn name(value: &str) -> PolicyName {
        PolicyName::try_from(value.to_string()).expect("policy name")
    }

    fn static_policy(policy_name: &str, source: &str) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            name: name(policy_name),
            description: None,
            origin: PolicyOrigin::Custom,
            enabled: true,
            body: PolicyBody::Static {
                source: source.to_string(),
            },
        }
    }

    fn template(template_name: &str, source: &str) -> PolicyTemplate {
        PolicyTemplate {
            id: Uuid::new_v4(),
            name: name(template_name),
            description: None,
            source: source.to_string(),
        }
    }

    fn linked(
        policy_name: &str,
        template: &PolicyTemplate,
        links: &[(SlotId, &str, &str)],
    ) -> Policy {
        Policy {
            id: Uuid::new_v4(),
            name: name(policy_name),
            description: None,
            origin: PolicyOrigin::Generated,
            enabled: true,
            body: PolicyBody::Linked {
                template_id: template.id,
                links: links
                    .iter()
                    .map(|(slot, ty, id)| {
                        (
                            *slot,
                            EntityRef::new(EntityType::custom(ty.to_string()), *id),
                        )
                    })
                    .collect::<BTreeMap<_, _>>(),
            },
        }
    }

    fn compiler() -> CedarPolicyCompiler {
        CedarPolicyCompiler::new()
    }

    const READ_ROLE_TEMPLATE: &str = r#"permit (
  principal in ?principal,
  action == App::Action::"document:read",
  resource
);"#;

    #[test]
    fn a_tenant_schema_in_its_namespace_may_reference_ferriskey_types() {
        assert_eq!(compiler().validate_schema(&schema(APP_SCHEMA)), Ok(()));
    }

    #[test]
    fn a_tenant_schema_cannot_declare_outside_its_namespace() {
        for source in [
            "namespace FerrisKey { entity Document; }",
            "namespace Billing { entity Invoice; }",
            "entity Document;",
        ] {
            let issues = compiler()
                .validate_schema(&schema(source))
                .expect_err(source);
            assert!(!issues.is_empty(), "{source}");
        }
    }

    #[test]
    fn a_schema_syntax_error_points_at_its_line_and_column() {
        let issues = compiler()
            .validate_schema(&schema(
                "namespace App {\n  entity Document = {\n    owner: ,\n  };\n}",
            ))
            .expect_err("syntax error");
        let first = &issues[0];
        assert_eq!(first.policy, None);
        assert_eq!(first.line, Some(3));
        assert!(first.column.is_some(), "{first:?}");
    }

    #[test]
    fn a_policy_using_an_undeclared_type_is_refused_by_name_and_position() {
        let policy = static_policy(
            "folders",
            "permit (\n  principal,\n  action,\n  resource is App::Folder\n);",
        );
        let issues = compiler()
            .validate_policies(Some(&schema(APP_SCHEMA)), &[policy], &[])
            .expect_err("undeclared type");
        let first = &issues[0];
        assert_eq!(first.policy.as_deref(), Some("folders"));
        assert_eq!(first.line, Some(4));
        assert!(first.column.is_some(), "{first:?}");
    }

    #[test]
    fn without_a_schema_only_the_syntax_is_checked() {
        let well_formed = static_policy(
            "any",
            "permit (principal, action, resource is App::Folder);",
        );
        assert_eq!(
            compiler().validate_policies(None, &[well_formed], &[]),
            Ok(())
        );

        let malformed = static_policy("broken", "permit (principal, action resource);");
        let issues = compiler()
            .validate_policies(None, &[malformed], &[])
            .expect_err("malformed");
        assert_eq!(issues[0].policy.as_deref(), Some("broken"));
    }

    #[test]
    fn a_static_policy_cannot_hold_a_slot() {
        let policy = static_policy("slotted", READ_ROLE_TEMPLATE);
        assert!(compiler().validate_policies(None, &[policy], &[]).is_err());
    }

    #[test]
    fn a_linked_policy_fills_exactly_the_slots_of_its_template() {
        let readers = template("readers", READ_ROLE_TEMPLATE);
        let good = linked(
            "editors-read",
            &readers,
            &[(SlotId::Principal, "FerrisKey::Role", "6b1f…")],
        );
        assert_eq!(
            compiler().validate_policies(
                Some(&schema(APP_SCHEMA)),
                &[good],
                std::slice::from_ref(&readers)
            ),
            Ok(())
        );

        let missing = linked("nobody", &readers, &[]);
        let extra = linked(
            "too-many",
            &readers,
            &[
                (SlotId::Principal, "FerrisKey::Role", "r1"),
                (SlotId::Resource, "App::Document", "d1"),
            ],
        );
        let unparsable = linked(
            "bad-type",
            &readers,
            &[(SlotId::Principal, "not a type", "r1")],
        );
        for policy in [missing, extra, unparsable] {
            let label = policy.name.to_string();
            let issues = compiler()
                .validate_policies(None, &[policy], std::slice::from_ref(&readers))
                .expect_err(&label);
            assert_eq!(issues[0].policy.as_deref(), Some(label.as_str()));
        }
    }

    #[test]
    fn a_template_using_an_undeclared_type_is_refused_by_name() {
        let folders = template(
            "folder-readers",
            "permit (principal in ?principal, action, resource is App::Folder);",
        );
        let issues = compiler()
            .validate_policies(Some(&schema(APP_SCHEMA)), &[], &[folders])
            .expect_err("undeclared type");
        assert_eq!(issues[0].policy.as_deref(), Some("folder-readers"));
    }

    #[test]
    fn a_link_to_an_undeclared_type_is_refused_against_a_schema() {
        let readers = template("readers", READ_ROLE_TEMPLATE);
        let policy = linked(
            "ghosts",
            &readers,
            &[(SlotId::Principal, "App::Ghost", "g1")],
        );
        let issues = compiler()
            .validate_policies(
                Some(&schema(APP_SCHEMA)),
                &[policy],
                std::slice::from_ref(&readers),
            )
            .expect_err("undeclared link type");
        assert_eq!(issues[0].policy.as_deref(), Some("ghosts"));
    }

    #[test]
    fn a_warning_does_not_block() {
        let policy = static_policy(
            "impossible",
            r#"permit (principal is App::Document, action == App::Action::"document:read", resource);"#,
        );
        let app = schema(APP_SCHEMA);

        let (set, _) = build_policy_set(std::slice::from_ref(&policy), &[]).expect("policy set");
        let compiled = compile_schema(&app).expect("schema");
        let result = Validator::new(compiled).validate(&set, ValidationMode::Strict);
        assert!(result.validation_warnings().next().is_some());

        assert_eq!(
            compiler().validate_policies(Some(&app), &[policy], &[]),
            Ok(())
        );
    }
}
