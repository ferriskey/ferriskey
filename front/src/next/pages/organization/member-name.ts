import { isServiceAccount } from '@/utils'
import { Schemas } from '@/api/api.client'

import User = Schemas.User

export const memberDisplayName = (user: User) =>
  isServiceAccount(user)
    ? 'Service Account'
    : [user.firstname, user.lastname].filter(Boolean).join(' ') || user.username
