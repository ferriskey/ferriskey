import { Schemas } from '@/api/api.client'
import { Client, User, Role, RealmSetting } from './api.types'

export interface PageHomeData {
  clients: Client[]
  users: User[]
  roles: Role[]
  flowStats: Schemas.FlowStats | null
  realmSettings: RealmSetting | null
  isLoading: boolean
}

export interface QuickAccessItem {
  title: string
  icon: React.ComponentType<{ className?: string }>
  url: string
  description: string
}
