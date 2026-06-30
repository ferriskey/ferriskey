import { REALM_SETTINGS_URL, REALM_URL } from '../router'

const CONSOLE_BRANDING_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/console/branding`

export const EMAIL_TEMPLATES_URL = (realmName = ':realmName') =>
  `${CONSOLE_BRANDING_URL(realmName)}/email-templates`

export const EMAIL_TEMPLATE_URL = (realmName = ':realmName', templateId = ':templateId') =>
  `${EMAIL_TEMPLATES_URL(realmName)}/${templateId}`

export const EMAIL_TEMPLATE_BUILDER_URL = (realmName = ':realmName', templateId = ':templateId') =>
  `${EMAIL_TEMPLATE_URL(realmName, templateId)}/builder`

// Admin (IAM) — email templates also live under the realm root so the builder
// renders inside the admin Layout instead of the console ProductLayout.
export const ADMIN_EMAIL_TEMPLATES_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/email-templates`

export const ADMIN_EMAIL_TEMPLATE_BUILDER_URL = (
  realmName = ':realmName',
  templateId = ':templateId',
) => `${ADMIN_EMAIL_TEMPLATES_URL(realmName)}/${templateId}/builder`

/**
 * Where the email builder returns to (Back / after save), derived from the
 * current path: the console branding list in CIAM mode, otherwise the admin
 * Realm Settings → Email tab.
 */
export const emailTemplatesListUrl = (pathname: string, realmName = ':realmName') =>
  pathname.includes('/console/')
    ? EMAIL_TEMPLATES_URL(realmName)
    : `${REALM_SETTINGS_URL(realmName)}/email`

export type EmailTemplateRouterParams = {
  realm_name: string
  template_id: string
}
