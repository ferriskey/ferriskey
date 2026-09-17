import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'
import { preloadNamespaces, translate } from '@/lib/i18n'

import User = Schemas.User

export const ORGANIZATION_NAMESPACE = 'organization'

void preloadNamespaces(ORGANIZATION_NAMESPACE).catch(() => undefined)

export const memberDisplayName = (user: User) =>
  isServiceAccount(user)
    ? translate(`${ORGANIZATION_NAMESPACE}:member.service_account`)
    : [user.firstname, user.lastname].filter(Boolean).join(' ') || user.username
