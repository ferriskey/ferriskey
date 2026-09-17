// ─── Field definitions ────────────────────────────────────────────────────────

export type ConfigFieldType = 'text' | 'select' | 'switch'

export type SelectOption = {
  labelKey: string
  value: string
}

export type ConfigFieldDef = {
  /** Key used in the mapper's config object (may contain dots) */
  key: string
  labelKey: string
  type: ConfigFieldType
  placeholder?: string
  options?: SelectOption[]
  defaultValue?: string
  descriptionKey?: string
}

// ─── Shared field helpers ─────────────────────────────────────────────────────

const CLAIM_JSON_TYPE_FIELD: ConfigFieldDef = {
  key: 'claim.value.type',
  labelKey: 'mapper_field.claim_value_type.label',
  type: 'select',
  defaultValue: 'String',
  options: [
    { labelKey: 'mapper_field.claim_value_type.options.string', value: 'String' },
    { labelKey: 'mapper_field.claim_value_type.options.json', value: 'JSON' },
    { labelKey: 'mapper_field.claim_value_type.options.long', value: 'long' },
    { labelKey: 'mapper_field.claim_value_type.options.int', value: 'int' },
    { labelKey: 'mapper_field.claim_value_type.options.boolean', value: 'boolean' },
  ],
}

const TOKEN_INCLUSION_FIELDS: ConfigFieldDef[] = [
  {
    key: 'id.token.claim',
    labelKey: 'mapper_field.id_token_claim.label',
    type: 'switch',
    defaultValue: 'true',
  },
  {
    key: 'access.token.claim',
    labelKey: 'mapper_field.access_token_claim.label',
    type: 'switch',
    defaultValue: 'true',
  },
  {
    key: 'userinfo.token.claim',
    labelKey: 'mapper_field.userinfo_token_claim.label',
    type: 'switch',
    defaultValue: 'true',
  },
  {
    key: 'introspection.token.claim',
    labelKey: 'mapper_field.introspection_token_claim.label',
    type: 'switch',
    defaultValue: 'false',
  },
]

// ─── Template type ────────────────────────────────────────────────────────────

export type MapperTemplate = {
  id: string
  nameKey: string
  descriptionKey: string
  icon: string
  mapper_type: string
  defaultName: string
  fields: ConfigFieldDef[]
  isCustom?: true
}

// ─── Quick Start (opinionated presets with pre-filled defaults) ───────────────

export const QUICK_START_TEMPLATES: MapperTemplate[] = [
  {
    id: 'qs-email',
    nameKey: 'mapper_template.qs-email.name',
    descriptionKey: 'mapper_template.qs-email.description',
    icon: '📧',
    mapper_type: 'oidc-usermodel-property-mapper',
    defaultName: 'email',
    fields: [
      {
        key: 'user.attribute',
        labelKey: 'mapper_field.user_property.label',
        type: 'text',
        placeholder: 'email',
        defaultValue: 'email',
      },
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'email',
        defaultValue: 'email',
      },
      CLAIM_JSON_TYPE_FIELD,
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'qs-username',
    nameKey: 'mapper_template.qs-username.name',
    descriptionKey: 'mapper_template.qs-username.description',
    icon: '👤',
    mapper_type: 'oidc-usermodel-property-mapper',
    defaultName: 'username',
    fields: [
      {
        key: 'user.attribute',
        labelKey: 'mapper_field.user_property.label',
        type: 'text',
        placeholder: 'username',
        defaultValue: 'username',
      },
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'preferred_username',
        defaultValue: 'preferred_username',
      },
      CLAIM_JSON_TYPE_FIELD,
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'qs-full-name',
    nameKey: 'mapper_template.qs-full-name.name',
    descriptionKey: 'mapper_template.qs-full-name.description',
    icon: '🪪',
    mapper_type: 'oidc-full-name-mapper',
    defaultName: 'full name',
    fields: TOKEN_INCLUSION_FIELDS,
  },
  {
    id: 'qs-realm-roles',
    nameKey: 'mapper_template.qs-realm-roles.name',
    descriptionKey: 'mapper_template.qs-realm-roles.description',
    icon: '🔐',
    mapper_type: 'oidc-usermodel-realm-role-mapper',
    defaultName: 'realm roles',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'roles',
        defaultValue: 'roles',
      },
      CLAIM_JSON_TYPE_FIELD,
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'qs-audience',
    nameKey: 'mapper_template.qs-audience.name',
    descriptionKey: 'mapper_template.qs-audience.description',
    icon: '🔑',
    mapper_type: 'oidc-audience-mapper',
    defaultName: 'audience',
    fields: [
      {
        key: 'included.custom.audience',
        labelKey: 'mapper_field.custom_audience.label',
        type: 'text',
        placeholder: 'https://api.example.com',
        defaultValue: '',
      },
      {
        key: 'id.token.claim',
        labelKey: 'mapper_field.id_token_claim.label',
        type: 'switch',
        defaultValue: 'false',
      },
      {
        key: 'access.token.claim',
        labelKey: 'mapper_field.access_token_claim.label',
        type: 'switch',
        defaultValue: 'true',
      },
    ],
  },
  {
    id: 'qs-org-membership',
    nameKey: 'mapper_template.qs-org-membership.name',
    descriptionKey: 'mapper_template.qs-org-membership.description',
    icon: '🏢',
    mapper_type: 'oidc-organization-membership-mapper',
    defaultName: 'organizations',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.claim_name.label',
        type: 'text',
        placeholder: 'organizations',
        defaultValue: 'organizations',
      },
      {
        key: 'include.domain',
        labelKey: 'mapper_field.include_domain.label',
        type: 'switch',
        defaultValue: 'false',
      },
      {
        key: 'include.attributes',
        labelKey: 'mapper_field.include_attributes.label',
        type: 'switch',
        defaultValue: 'false',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'qs-org-role',
    nameKey: 'mapper_template.qs-org-role.name',
    descriptionKey: 'mapper_template.qs-org-role.description',
    icon: '🛡️',
    mapper_type: 'oidc-organization-role-mapper',
    defaultName: 'organization_roles',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.claim_name.label',
        type: 'text',
        placeholder: 'organizations',
        defaultValue: 'organizations',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'qs-org-detail',
    nameKey: 'mapper_template.qs-org-detail.name',
    descriptionKey: 'mapper_template.qs-org-detail.description',
    icon: '🏛️',
    mapper_type: 'oidc-organization-detail-mapper',
    defaultName: 'organization',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.claim_name.label',
        type: 'text',
        placeholder: 'organization',
        defaultValue: 'organization',
      },
      {
        key: 'organization.alias',
        labelKey: 'mapper_field.organization_alias.label',
        type: 'text',
        placeholder: 'acme',
        defaultValue: '',
        descriptionKey: 'mapper_field.organization_alias.description_short',
      },
      {
        key: 'include.attributes',
        labelKey: 'mapper_field.include_attributes.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.include_attributes.description_claim',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
]

// ─── By-configuration catalog (full field set, no opinionated defaults) ───────

export const MAPPER_CATALOG: MapperTemplate[] = [
  {
    id: 'cat-role-name',
    nameKey: 'mapper_template.cat-role-name.name',
    descriptionKey: 'mapper_template.cat-role-name.description',
    icon: '🏷️',
    mapper_type: 'oidc-role-name-mapper',
    defaultName: '',
    fields: [
      {
        key: 'role',
        labelKey: 'mapper_field.role.label',
        type: 'text',
        placeholder: 'my-role  or  client-id.role-name',
        defaultValue: '',
        descriptionKey: 'mapper_field.role.description_rename',
      },
      {
        key: 'new.role.name',
        labelKey: 'mapper_field.new_role_name.label',
        type: 'text',
        placeholder: 'custom-name',
        defaultValue: '',
        descriptionKey: 'mapper_field.new_role_name.description',
      },
    ],
  },
  {
    id: 'cat-user-attribute',
    nameKey: 'mapper_template.cat-user-attribute.name',
    descriptionKey: 'mapper_template.cat-user-attribute.description',
    icon: '🗂️',
    mapper_type: 'oidc-usermodel-attribute-mapper',
    defaultName: '',
    fields: [
      {
        key: 'user.attribute',
        labelKey: 'mapper_field.user_attribute.label',
        type: 'text',
        placeholder: 'my-attribute',
        defaultValue: '',
        descriptionKey: 'mapper_field.user_attribute.description',
      },
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'my-claim',
        defaultValue: '',
        descriptionKey: 'mapper_field.token_claim_name.description_nested',
      },
      CLAIM_JSON_TYPE_FIELD,
      {
        key: 'multivalued',
        labelKey: 'mapper_field.multivalued.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.multivalued.description_attribute',
      },
      {
        key: 'aggregate.attrs',
        labelKey: 'mapper_field.aggregate_attrs.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.aggregate_attrs.description',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-user-property',
    nameKey: 'mapper_template.cat-user-property.name',
    descriptionKey: 'mapper_template.cat-user-property.description',
    icon: '👤',
    mapper_type: 'oidc-usermodel-property-mapper',
    defaultName: '',
    fields: [
      {
        key: 'user.attribute',
        labelKey: 'mapper_field.user_property_builtin.label',
        type: 'text',
        placeholder: 'email  or  username  or  firstName',
        defaultValue: '',
        descriptionKey: 'mapper_field.user_property_builtin.description',
      },
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'my-claim',
        defaultValue: '',
      },
      CLAIM_JSON_TYPE_FIELD,
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-user-client-role',
    nameKey: 'mapper_template.cat-user-client-role.name',
    descriptionKey: 'mapper_template.cat-user-client-role.description',
    icon: '🎭',
    mapper_type: 'oidc-usermodel-client-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'client.id',
        labelKey: 'mapper_field.client_id.label',
        type: 'text',
        placeholder: 'my-client',
        defaultValue: '',
        descriptionKey: 'mapper_field.client_id.description',
      },
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'resource_access.${client_id}.roles',
        defaultValue: '',
      },
      CLAIM_JSON_TYPE_FIELD,
      {
        key: 'multivalued',
        labelKey: 'mapper_field.multivalued.label',
        type: 'switch',
        defaultValue: 'true',
        descriptionKey: 'mapper_field.multivalued.description_roles',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-realm-role',
    nameKey: 'mapper_template.cat-realm-role.name',
    descriptionKey: 'mapper_template.cat-realm-role.description',
    icon: '🔐',
    mapper_type: 'oidc-usermodel-realm-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'roles',
        defaultValue: '',
      },
      CLAIM_JSON_TYPE_FIELD,
      {
        key: 'multivalued',
        labelKey: 'mapper_field.multivalued.label',
        type: 'switch',
        defaultValue: 'true',
        descriptionKey: 'mapper_field.multivalued.description_roles',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-group-membership',
    nameKey: 'mapper_template.cat-group-membership.name',
    descriptionKey: 'mapper_template.cat-group-membership.description',
    icon: '👥',
    mapper_type: 'oidc-group-membership-mapper',
    defaultName: '',
    fields: [
      {
        key: 'token.claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'groups',
        defaultValue: '',
      },
      {
        key: 'membership',
        labelKey: 'mapper_field.membership.label',
        type: 'select',
        defaultValue: 'effective',
        options: [
          { labelKey: 'mapper_field.membership.options.effective', value: 'effective' },
          { labelKey: 'mapper_field.membership.options.direct', value: 'direct' },
        ],
        descriptionKey: 'mapper_field.membership.description',
      },
      {
        key: 'full.path',
        labelKey: 'mapper_field.full_path.label',
        type: 'switch',
        defaultValue: 'true',
        descriptionKey: 'mapper_field.full_path.description',
      },
      {
        key: 'prefix.org',
        labelKey: 'mapper_field.prefix_org.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.prefix_org.description',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-audience',
    nameKey: 'mapper_template.cat-audience.name',
    descriptionKey: 'mapper_template.cat-audience.description',
    icon: '🔑',
    mapper_type: 'oidc-audience-mapper',
    defaultName: '',
    fields: [
      {
        key: 'included.client.audience',
        labelKey: 'mapper_field.client_audience.label',
        type: 'text',
        placeholder: 'my-client',
        defaultValue: '',
        descriptionKey: 'mapper_field.client_audience.description',
      },
      {
        key: 'included.custom.audience',
        labelKey: 'mapper_field.custom_audience.label',
        type: 'text',
        placeholder: 'https://api.example.com',
        defaultValue: '',
        descriptionKey: 'mapper_field.custom_audience.description',
      },
      {
        key: 'id.token.claim',
        labelKey: 'mapper_field.id_token_claim.label',
        type: 'switch',
        defaultValue: 'false',
      },
      {
        key: 'access.token.claim',
        labelKey: 'mapper_field.access_token_claim.label',
        type: 'switch',
        defaultValue: 'true',
      },
    ],
  },
  {
    id: 'cat-hardcoded-claim',
    nameKey: 'mapper_template.cat-hardcoded-claim.name',
    descriptionKey: 'mapper_template.cat-hardcoded-claim.description',
    icon: '📌',
    mapper_type: 'oidc-hardcoded-claim-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.token_claim_name.label',
        type: 'text',
        placeholder: 'my-claim',
        defaultValue: '',
      },
      {
        key: 'claim.value',
        labelKey: 'mapper_field.claim_value.label',
        type: 'text',
        placeholder: 'my-value',
        defaultValue: '',
      },
      CLAIM_JSON_TYPE_FIELD,
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-hardcoded-role',
    nameKey: 'mapper_template.cat-hardcoded-role.name',
    descriptionKey: 'mapper_template.cat-hardcoded-role.description',
    icon: '🔒',
    mapper_type: 'oidc-hardcoded-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'role',
        labelKey: 'mapper_field.role.label',
        type: 'text',
        placeholder: 'my-role',
        defaultValue: '',
        descriptionKey: 'mapper_field.role.description_hardcoded',
      },
    ],
  },
  {
    id: 'cat-address',
    nameKey: 'mapper_template.cat-address.name',
    descriptionKey: 'mapper_template.cat-address.description',
    icon: '🏠',
    mapper_type: 'oidc-address-mapper',
    defaultName: '',
    fields: [
      {
        key: 'user.attribute.formatted',
        labelKey: 'mapper_field.address_formatted.label',
        type: 'text',
        placeholder: 'formatted',
        defaultValue: '',
      },
      {
        key: 'user.attribute.street',
        labelKey: 'mapper_field.address_street.label',
        type: 'text',
        placeholder: 'street',
        defaultValue: '',
      },
      {
        key: 'user.attribute.locality',
        labelKey: 'mapper_field.address_locality.label',
        type: 'text',
        placeholder: 'locality',
        defaultValue: '',
      },
      {
        key: 'user.attribute.region',
        labelKey: 'mapper_field.address_region.label',
        type: 'text',
        placeholder: 'region',
        defaultValue: '',
      },
      {
        key: 'user.attribute.postal_code',
        labelKey: 'mapper_field.address_postal_code.label',
        type: 'text',
        placeholder: 'postal_code',
        defaultValue: '',
      },
      {
        key: 'user.attribute.country',
        labelKey: 'mapper_field.address_country.label',
        type: 'text',
        placeholder: 'country',
        defaultValue: '',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-org-membership',
    nameKey: 'mapper_template.cat-org-membership.name',
    descriptionKey: 'mapper_template.cat-org-membership.description',
    icon: '🏢',
    mapper_type: 'oidc-organization-membership-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.claim_name.label',
        type: 'text',
        placeholder: 'organizations',
        defaultValue: 'organizations',
        descriptionKey: 'mapper_field.claim_name.description_organization_list',
      },
      {
        key: 'include.domain',
        labelKey: 'mapper_field.include_domain.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.include_domain.description',
      },
      {
        key: 'include.attributes',
        labelKey: 'mapper_field.include_attributes.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.include_attributes.description_entry',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-org-role',
    nameKey: 'mapper_template.cat-org-role.name',
    descriptionKey: 'mapper_template.cat-org-role.description',
    icon: '🛡️',
    mapper_type: 'oidc-organization-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.claim_name.label',
        type: 'text',
        placeholder: 'organizations',
        defaultValue: 'organizations',
        descriptionKey: 'mapper_field.claim_name.description_organization_root',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-org-detail',
    nameKey: 'mapper_template.cat-org-detail.name',
    descriptionKey: 'mapper_template.cat-org-detail.description',
    icon: '🏛️',
    mapper_type: 'oidc-organization-detail-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        labelKey: 'mapper_field.claim_name.label',
        type: 'text',
        placeholder: 'organization',
        defaultValue: 'organization',
        descriptionKey: 'mapper_field.claim_name.description_organization_object',
      },
      {
        key: 'organization.alias',
        labelKey: 'mapper_field.organization_alias.label',
        type: 'text',
        placeholder: 'acme',
        defaultValue: '',
        descriptionKey: 'mapper_field.organization_alias.description_long',
      },
      {
        key: 'include.attributes',
        labelKey: 'mapper_field.include_attributes.label',
        type: 'switch',
        defaultValue: 'false',
        descriptionKey: 'mapper_field.include_attributes.description_claim',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-custom',
    nameKey: 'mapper_template.cat-custom.name',
    descriptionKey: 'mapper_template.cat-custom.description',
    icon: '⚙️',
    mapper_type: '',
    defaultName: '',
    fields: [],
    isCustom: true,
  },
]

// ─── Combined lookup (used by the create page to resolve ?template=id) ────────

export const ALL_MAPPER_TEMPLATES: MapperTemplate[] = [...QUICK_START_TEMPLATES, ...MAPPER_CATALOG]

// Keep backward-compatible alias used by existing imports
export const PROTOCOL_MAPPER_TEMPLATES = ALL_MAPPER_TEMPLATES

export const FALLBACK_MAPPER_ICON = '⚙️'
