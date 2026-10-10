import type { Schemas } from './api.client'

// TODO: remove this file once api.client.ts is regenerated with dynamic_registration_allowed.
export interface DynamicRegistrationFlag {
  dynamic_registration_allowed?: boolean
}

export type ClientScopeWithDynamic = Schemas.ClientScope & DynamicRegistrationFlag
export type CreateClientScopeBody = Schemas.CreateClientScopeValidator & DynamicRegistrationFlag
export type UpdateClientScopeBody = Schemas.UpdateClientScopeValidator & DynamicRegistrationFlag
