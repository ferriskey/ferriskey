import { Realm } from './core.interface'
import { Schemas } from './api.client'

export const mapRealms = (realm: Schemas.Realm[]): Realm[] => {
  return realm.map((item) => ({
    id: item.id,
    name: item.name,
    created_at: new Date(item.created_at),
    updated_at: new Date(item.updated_at),
  }))
}
