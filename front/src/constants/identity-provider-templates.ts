export interface ProviderTemplate {
  id: string
  name: string
  displayName: string
  description: string
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
    label: string
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
    description: 'Sign in with Google accounts',
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
    description: 'Sign in with Discord accounts',
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
    description: 'Sign in with GitHub accounts',
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
    description: 'Sign in with Microsoft & Azure AD',
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
    description: 'Sign in with Apple ID',
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
    description: 'Sign in with Facebook accounts',
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
    description: 'Sign in with GitLab accounts',
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
    description: 'Sign in with X/Twitter accounts',
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
    description: 'Sign in with LinkedIn accounts',
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
  description: 'Configure a custom OAuth2/OIDC provider',
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

export function getTemplatesByCategory(category: ProviderTemplate['category']): ProviderTemplate[] {
  return PROVIDER_TEMPLATES.filter((t) => t.category === category)
}
