import { preloadNamespaces, translate } from '@/lib/i18n'

export const IDENTITY_PROVIDER_NAMESPACE = 'identity-provider'

void preloadNamespaces(IDENTITY_PROVIDER_NAMESPACE).catch(() => undefined)

export interface ProviderTemplate {
  id: string
  name: string
  displayName: string
  displayNameKey?: string
  descriptionKey: string
  icon: 'google' | 'discord' | 'github' | 'microsoft' | 'apple' | 'facebook' | 'gitlab' | 'twitter' | 'linkedin' | 'custom'
  category: 'social' | 'enterprise' | 'developer' | 'custom'
  provider_type: 'oidc' | 'oauth2'
  documentation_url: string
  authorization_url: string
  token_url: string
  userinfo_url?: string
  default_scopes: string[]
  custom_fields?: Array<{
    name: string
    labelKey: string
    placeholder: string
    required: boolean
    type: 'text' | 'url' | 'select'
    options?: string[]
  }>
}

export const PROVIDER_TEMPLATES: ProviderTemplate[] = [
  {
    id: 'google',
    name: 'google',
    displayName: 'Google',
    descriptionKey: 'template.google.description',
    icon: 'google',
    category: 'social',
    provider_type: 'oidc',
    documentation_url: 'https://developers.google.com/identity/protocols/oauth2',
    authorization_url: 'https://accounts.google.com/o/oauth2/v2/auth',
    token_url: 'https://oauth2.googleapis.com/token',
    userinfo_url: 'https://www.googleapis.com/oauth2/v3/userinfo',
    default_scopes: ['openid', 'email', 'profile'],
  },
  {
    id: 'discord',
    name: 'discord',
    displayName: 'Discord',
    descriptionKey: 'template.discord.description',
    icon: 'discord',
    category: 'social',
    provider_type: 'oauth2',
    documentation_url: 'https://discord.com/developers/docs/topics/oauth2',
    authorization_url: 'https://discord.com/api/oauth2/authorize',
    token_url: 'https://discord.com/api/oauth2/token',
    userinfo_url: 'https://discord.com/api/users/@me',
    default_scopes: ['identify', 'email'],
  },
  {
    id: 'github',
    name: 'github',
    displayName: 'GitHub',
    descriptionKey: 'template.github.description',
    icon: 'github',
    category: 'developer',
    provider_type: 'oauth2',
    documentation_url: 'https://docs.github.com/en/apps/oauth-apps/building-oauth-apps',
    authorization_url: 'https://github.com/login/oauth/authorize',
    token_url: 'https://github.com/login/oauth/access_token',
    userinfo_url: 'https://api.github.com/user',
    default_scopes: ['read:user', 'user:email'],
  },
  {
    id: 'microsoft',
    name: 'microsoft',
    displayName: 'Microsoft',
    descriptionKey: 'template.microsoft.description',
    icon: 'microsoft',
    category: 'enterprise',
    provider_type: 'oidc',
    documentation_url: 'https://learn.microsoft.com/en-us/entra/identity-platform/',
    authorization_url: 'https://login.microsoftonline.com/common/oauth2/v2.0/authorize',
    token_url: 'https://login.microsoftonline.com/common/oauth2/v2.0/token',
    userinfo_url: 'https://graph.microsoft.com/v1.0/me',
    default_scopes: ['openid', 'profile', 'email'],
  },
  {
    id: 'apple',
    name: 'apple',
    displayName: 'Apple',
    descriptionKey: 'template.apple.description',
    icon: 'apple',
    category: 'social',
    provider_type: 'oidc',
    documentation_url: 'https://developer.apple.com/sign-in-with-apple/',
    authorization_url: 'https://appleid.apple.com/auth/authorize',
    token_url: 'https://appleid.apple.com/auth/token',
    default_scopes: ['name', 'email'],
  },
  {
    id: 'facebook',
    name: 'facebook',
    displayName: 'Facebook',
    descriptionKey: 'template.facebook.description',
    icon: 'facebook',
    category: 'social',
    provider_type: 'oauth2',
    documentation_url: 'https://developers.facebook.com/docs/facebook-login/',
    authorization_url: 'https://www.facebook.com/v18.0/dialog/oauth',
    token_url: 'https://graph.facebook.com/v18.0/oauth/access_token',
    userinfo_url: 'https://graph.facebook.com/me?fields=id,name,email,picture',
    default_scopes: ['email', 'public_profile'],
  },
  {
    id: 'gitlab',
    name: 'gitlab',
    displayName: 'GitLab',
    descriptionKey: 'template.gitlab.description',
    icon: 'gitlab',
    category: 'developer',
    provider_type: 'oidc',
    documentation_url: 'https://docs.gitlab.com/ee/integration/oauth_provider.html',
    authorization_url: 'https://gitlab.com/oauth/authorize',
    token_url: 'https://gitlab.com/oauth/token',
    userinfo_url: 'https://gitlab.com/api/v4/user',
    default_scopes: ['openid', 'profile', 'email'],
  },
  {
    id: 'twitter',
    name: 'twitter',
    displayName: 'X (Twitter)',
    descriptionKey: 'template.twitter.description',
    icon: 'twitter',
    category: 'social',
    provider_type: 'oauth2',
    documentation_url: 'https://developer.twitter.com/en/docs/authentication/oauth-2-0',
    authorization_url: 'https://twitter.com/i/oauth2/authorize',
    token_url: 'https://api.twitter.com/2/oauth2/token',
    userinfo_url: 'https://api.twitter.com/2/users/me',
    default_scopes: ['users.read', 'tweet.read'],
  },
  {
    id: 'linkedin',
    name: 'linkedin',
    displayName: 'LinkedIn',
    descriptionKey: 'template.linkedin.description',
    icon: 'linkedin',
    category: 'enterprise',
    provider_type: 'oidc',
    documentation_url: 'https://learn.microsoft.com/en-us/linkedin/shared/authentication/authorization-code-flow',
    authorization_url: 'https://www.linkedin.com/oauth/v2/authorization',
    token_url: 'https://www.linkedin.com/oauth/v2/accessToken',
    userinfo_url: 'https://api.linkedin.com/v2/userinfo',
    default_scopes: ['openid', 'profile', 'email'],
  },
]

export const CUSTOM_PROVIDER_TEMPLATE: ProviderTemplate = {
  id: 'custom',
  name: 'custom',
  displayName: 'Custom Provider',
  displayNameKey: 'template.custom.display_name',
  descriptionKey: 'template.custom.description',
  icon: 'custom',
  category: 'custom',
  provider_type: 'oauth2',
  documentation_url: '',
  authorization_url: '',
  token_url: '',
  userinfo_url: '',
  default_scopes: [],
}

export const ALL_TEMPLATES = [...PROVIDER_TEMPLATES, CUSTOM_PROVIDER_TEMPLATE]

export function getTemplateById(id: string): ProviderTemplate | undefined {
  return ALL_TEMPLATES.find((t) => t.id === id)
}

export function templateDisplayName(template: ProviderTemplate): string {
  return template.displayNameKey
    ? translate(`${IDENTITY_PROVIDER_NAMESPACE}:${template.displayNameKey}`)
    : template.displayName
}

export function templateDescription(template: ProviderTemplate): string {
  return translate(`${IDENTITY_PROVIDER_NAMESPACE}:${template.descriptionKey}`)
}

export function getTemplatesByCategory(category: ProviderTemplate['category']): ProviderTemplate[] {
  return PROVIDER_TEMPLATES.filter((t) => t.category === category)
}
