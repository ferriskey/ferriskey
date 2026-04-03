// ─── Field definitions ────────────────────────────────────────────────────────

export type ConfigFieldType = 'text' | 'select' | 'switch'

export type SelectOption = {
  label: string
  value: string
}

export type ConfigFieldDef = {
  /** Key used in the mapper's config object (may contain dots) */
  key: string
  label: string
  type: ConfigFieldType
  placeholder?: string
  options?: SelectOption[]
  defaultValue?: string
  description?: string
}

// ─── Shared field helpers ─────────────────────────────────────────────────────

const CLAIM_JSON_TYPE_FIELD: ConfigFieldDef = {
  key: 'claim.value.type',
  label: 'Claim JSON type',
  type: 'select',
  defaultValue: 'String',
  options: [
    { label: 'String', value: 'String' },
    { label: 'JSON', value: 'JSON' },
    { label: 'Long', value: 'long' },
    { label: 'Integer', value: 'int' },
    { label: 'Boolean', value: 'boolean' },
  ],
}

const TOKEN_INCLUSION_FIELDS: ConfigFieldDef[] = [
  {
    key: 'id.token.claim',
    label: 'Add to ID token',
    type: 'switch',
    defaultValue: 'true',
  },
  {
    key: 'access.token.claim',
    label: 'Add to access token',
    type: 'switch',
    defaultValue: 'true',
  },
  {
    key: 'userinfo.token.claim',
    label: 'Add to userinfo',
    type: 'switch',
    defaultValue: 'true',
  },
  {
    key: 'introspection.token.claim',
    label: 'Add to token introspection',
    type: 'switch',
    defaultValue: 'false',
  },
]

// ─── Template type ────────────────────────────────────────────────────────────

export type MapperTemplate = {
  id: string
  name: string
  description: string
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
    name: 'Email',
    description: 'Maps the user email address to the email token claim',
    icon: '📧',
    mapper_type: 'oidc-usermodel-property-mapper',
    defaultName: 'email',
    fields: [
      {
        key: 'user.attribute',
        label: 'User property',
        type: 'text',
        placeholder: 'email',
        defaultValue: 'email',
      },
      {
        key: 'token.claim.name',
        label: 'Token claim name',
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
    name: 'Username',
    description: 'Maps username to the preferred_username claim',
    icon: '👤',
    mapper_type: 'oidc-usermodel-property-mapper',
    defaultName: 'username',
    fields: [
      {
        key: 'user.attribute',
        label: 'User property',
        type: 'text',
        placeholder: 'username',
        defaultValue: 'username',
      },
      {
        key: 'token.claim.name',
        label: 'Token claim name',
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
    name: 'Full Name',
    description: 'Maps the user full name to the name claim',
    icon: '🪪',
    mapper_type: 'oidc-full-name-mapper',
    defaultName: 'full name',
    fields: TOKEN_INCLUSION_FIELDS,
  },
  {
    id: 'qs-realm-roles',
    name: 'Realm Roles',
    description: 'Adds all realm roles as a multi-valued token claim',
    icon: '🔐',
    mapper_type: 'oidc-usermodel-realm-role-mapper',
    defaultName: 'realm roles',
    fields: [
      {
        key: 'token.claim.name',
        label: 'Token claim name',
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
    name: 'Audience',
    description: 'Adds a custom audience value to the token',
    icon: '🔑',
    mapper_type: 'oidc-audience-mapper',
    defaultName: 'audience',
    fields: [
      {
        key: 'included.custom.audience',
        label: 'Custom audience',
        type: 'text',
        placeholder: 'https://api.example.com',
        defaultValue: '',
      },
      {
        key: 'id.token.claim',
        label: 'Add to ID token',
        type: 'switch',
        defaultValue: 'false',
      },
      {
        key: 'access.token.claim',
        label: 'Add to access token',
        type: 'switch',
        defaultValue: 'true',
      },
    ],
  },
  {
    id: 'qs-org-membership',
    name: 'Organization Membership',
    description: 'Adds all organizations the user belongs to as a JSON array (supports domain & attributes)',
    icon: '🏢',
    mapper_type: 'oidc-organization-membership-mapper',
    defaultName: 'organizations',
    fields: [
      {
        key: 'claim.name',
        label: 'Claim name',
        type: 'text',
        placeholder: 'organizations',
        defaultValue: 'organizations',
      },
      {
        key: 'include.domain',
        label: 'Include domain',
        type: 'switch',
        defaultValue: 'false',
      },
      {
        key: 'include.attributes',
        label: 'Include attributes',
        type: 'switch',
        defaultValue: 'false',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'qs-org-detail',
    name: 'Organization Detail',
    description: 'Adds detailed information about a single organization to the token',
    icon: '🏛️',
    mapper_type: 'oidc-organization-detail-mapper',
    defaultName: 'organization',
    fields: [
      {
        key: 'claim.name',
        label: 'Claim name',
        type: 'text',
        placeholder: 'organization',
        defaultValue: 'organization',
      },
      {
        key: 'organization.alias',
        label: 'Organization alias filter',
        type: 'text',
        placeholder: 'acme',
        defaultValue: '',
        description: 'Leave empty to use the first organization the user belongs to.',
      },
      {
        key: 'include.attributes',
        label: 'Include attributes',
        type: 'switch',
        defaultValue: 'false',
        description: 'When enabled, organization custom attributes are added to the claim.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
]

// ─── By-configuration catalog (full field set, no opinionated defaults) ───────

export const MAPPER_CATALOG: MapperTemplate[] = [
  {
    id: 'cat-role-name',
    name: 'Role Name Mapper',
    description: 'Rename a role before it is injected into the token',
    icon: '🏷️',
    mapper_type: 'oidc-role-name-mapper',
    defaultName: '',
    fields: [
      {
        key: 'role',
        label: 'Role',
        type: 'text',
        placeholder: 'my-role  or  client-id.role-name',
        defaultValue: '',
        description:
          'The role to rename. Use "roleName" for realm roles, "clientId.roleName" for client roles.',
      },
      {
        key: 'new.role.name',
        label: 'New role name',
        type: 'text',
        placeholder: 'custom-name',
        defaultValue: '',
        description: 'The value this role will have inside the token.',
      },
    ],
  },
  {
    id: 'cat-user-attribute',
    name: 'User Attribute',
    description: 'Map any user attribute to a token claim',
    icon: '🗂️',
    mapper_type: 'oidc-usermodel-attribute-mapper',
    defaultName: '',
    fields: [
      {
        key: 'user.attribute',
        label: 'User attribute',
        type: 'text',
        placeholder: 'my-attribute',
        defaultValue: '',
        description: 'The name of the user attribute to map.',
      },
      {
        key: 'token.claim.name',
        label: 'Token claim name',
        type: 'text',
        placeholder: 'my-claim',
        defaultValue: '',
        description: 'Name of the claim as it will appear in the token.',
      },
      CLAIM_JSON_TYPE_FIELD,
      {
        key: 'multivalued',
        label: 'Multivalued',
        type: 'switch',
        defaultValue: 'false',
        description: 'Indicates if the attribute supports multiple values.',
      },
      {
        key: 'aggregate.attrs',
        label: 'Aggregate attribute values',
        type: 'switch',
        defaultValue: 'false',
        description: 'Merge values from the user and all their group memberships.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-user-property',
    name: 'User Property',
    description: 'Map a built-in user property (email, username…) to a token claim',
    icon: '👤',
    mapper_type: 'oidc-usermodel-property-mapper',
    defaultName: '',
    fields: [
      {
        key: 'user.attribute',
        label: 'Property',
        type: 'text',
        placeholder: 'email  or  username  or  firstName',
        defaultValue: '',
        description: 'Built-in property name on the user object.',
      },
      {
        key: 'token.claim.name',
        label: 'Token claim name',
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
    name: 'User Client Role',
    description: 'Map client-level roles for a specific client to a token claim',
    icon: '🎭',
    mapper_type: 'oidc-usermodel-client-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'client.id',
        label: 'Client ID',
        type: 'text',
        placeholder: 'my-client',
        defaultValue: '',
        description: 'Leave empty to include roles from all clients.',
      },
      {
        key: 'token.claim.name',
        label: 'Token claim name',
        type: 'text',
        placeholder: 'resource_access.${client_id}.roles',
        defaultValue: '',
      },
      CLAIM_JSON_TYPE_FIELD,
      {
        key: 'multivalued',
        label: 'Multivalued',
        type: 'switch',
        defaultValue: 'true',
        description: 'Roles are always multi-valued.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-realm-role',
    name: 'Realm Role',
    description: 'Map realm-level roles to a token claim with full control',
    icon: '🔐',
    mapper_type: 'oidc-usermodel-realm-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'token.claim.name',
        label: 'Token claim name',
        type: 'text',
        placeholder: 'roles',
        defaultValue: '',
      },
      CLAIM_JSON_TYPE_FIELD,
      {
        key: 'multivalued',
        label: 'Multivalued',
        type: 'switch',
        defaultValue: 'true',
        description: 'Roles are always multi-valued.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-group-membership',
    name: 'Group Membership',
    description: 'Map the user\'s group memberships to a token claim',
    icon: '👥',
    mapper_type: 'oidc-group-membership-mapper',
    defaultName: '',
    fields: [
      {
        key: 'token.claim.name',
        label: 'Token claim name',
        type: 'text',
        placeholder: 'groups',
        defaultValue: '',
      },
      {
        key: 'full.path',
        label: 'Full group path',
        type: 'switch',
        defaultValue: 'true',
        description: 'Include the full path (e.g. /parent/child) instead of just the group name.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-audience',
    name: 'Audience',
    description: 'Add a client or custom audience value to the token',
    icon: '🔑',
    mapper_type: 'oidc-audience-mapper',
    defaultName: '',
    fields: [
      {
        key: 'included.client.audience',
        label: 'Client audience',
        type: 'text',
        placeholder: 'my-client',
        defaultValue: '',
        description: 'Client whose audience should be added. Leave empty if using a custom value.',
      },
      {
        key: 'included.custom.audience',
        label: 'Custom audience',
        type: 'text',
        placeholder: 'https://api.example.com',
        defaultValue: '',
        description: 'A custom audience string to inject.',
      },
      {
        key: 'id.token.claim',
        label: 'Add to ID token',
        type: 'switch',
        defaultValue: 'false',
      },
      {
        key: 'access.token.claim',
        label: 'Add to access token',
        type: 'switch',
        defaultValue: 'true',
      },
    ],
  },
  {
    id: 'cat-hardcoded-claim',
    name: 'Hardcoded Claim',
    description: 'Inject a fixed static value into the token as a named claim',
    icon: '📌',
    mapper_type: 'oidc-hardcoded-claim-mapper',
    defaultName: '',
    fields: [
      {
        key: 'token.claim.name',
        label: 'Token claim name',
        type: 'text',
        placeholder: 'my-claim',
        defaultValue: '',
      },
      {
        key: 'claim.value',
        label: 'Claim value',
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
    name: 'Hardcoded Role',
    description: 'Always assign a specific role to the token, regardless of the user',
    icon: '🔒',
    mapper_type: 'oidc-hardcoded-role-mapper',
    defaultName: '',
    fields: [
      {
        key: 'role',
        label: 'Role',
        type: 'text',
        placeholder: 'my-role',
        defaultValue: '',
        description: 'The role to always inject. Use "roleName" or "clientId.roleName".',
      },
    ],
  },
  {
    id: 'cat-address',
    name: 'Address',
    description: 'Map the user\'s address attributes to the address claim',
    icon: '🏠',
    mapper_type: 'oidc-address-mapper',
    defaultName: '',
    fields: [
      {
        key: 'user.attribute.formatted',
        label: 'Formatted address attribute',
        type: 'text',
        placeholder: 'formatted',
        defaultValue: '',
      },
      {
        key: 'user.attribute.street',
        label: 'Street attribute',
        type: 'text',
        placeholder: 'street',
        defaultValue: '',
      },
      {
        key: 'user.attribute.locality',
        label: 'Locality (city) attribute',
        type: 'text',
        placeholder: 'locality',
        defaultValue: '',
      },
      {
        key: 'user.attribute.region',
        label: 'Region attribute',
        type: 'text',
        placeholder: 'region',
        defaultValue: '',
      },
      {
        key: 'user.attribute.postal_code',
        label: 'Postal code attribute',
        type: 'text',
        placeholder: 'postal_code',
        defaultValue: '',
      },
      {
        key: 'user.attribute.country',
        label: 'Country attribute',
        type: 'text',
        placeholder: 'country',
        defaultValue: '',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-org-membership',
    name: 'Organization Membership',
    description: 'Inject all organizations the user belongs to as a JSON array — works for multi-org users',
    icon: '🏢',
    mapper_type: 'oidc-organization-membership-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        label: 'Claim name',
        type: 'text',
        placeholder: 'organizations',
        defaultValue: 'organizations',
        description: 'Name of the token claim that will hold the organization list.',
      },
      {
        key: 'include.domain',
        label: 'Include domain',
        type: 'switch',
        defaultValue: 'false',
        description: 'Add the domain field to each organization entry (null when not set).',
      },
      {
        key: 'include.attributes',
        label: 'Include attributes',
        type: 'switch',
        defaultValue: 'false',
        description: 'Add organization custom attributes to each entry.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-org-detail',
    name: 'Organization Detail',
    description: 'Inject detailed information about a single organization into the token',
    icon: '🏛️',
    mapper_type: 'oidc-organization-detail-mapper',
    defaultName: '',
    fields: [
      {
        key: 'claim.name',
        label: 'Claim name',
        type: 'text',
        placeholder: 'organization',
        defaultValue: 'organization',
        description: 'Name of the token claim that will hold the organization object.',
      },
      {
        key: 'organization.alias',
        label: 'Organization alias filter',
        type: 'text',
        placeholder: 'acme',
        defaultValue: '',
        description:
          'Alias of the organization to emit. Leave empty to use the first organization the user belongs to.',
      },
      {
        key: 'include.attributes',
        label: 'Include attributes',
        type: 'switch',
        defaultValue: 'false',
        description: 'When enabled, organization custom attributes are added to the claim.',
      },
      ...TOKEN_INCLUSION_FIELDS,
    ],
  },
  {
    id: 'cat-custom',
    name: 'Custom mapper',
    description: 'Enter a mapper type and raw JSON configuration manually',
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
