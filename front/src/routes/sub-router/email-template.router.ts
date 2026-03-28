import { REALM_URL } from '../router'

export const EMAIL_TEMPLATES_URL = (realmName = ':realmName') =>
  `${REALM_URL(realmName)}/email-templates`

export const EMAIL_TEMPLATE_URL = (realmName = ':realmName', templateId = ':templateId') =>
  `${EMAIL_TEMPLATES_URL(realmName)}/${templateId}`

export const EMAIL_TEMPLATE_BUILDER_URL = (realmName = ':realmName', templateId = ':templateId') =>
  `${EMAIL_TEMPLATE_URL(realmName, templateId)}/builder`

export type EmailTemplateRouterParams = {
  realm_name: string
  template_id: string
}
