import { Schemas } from '@/api/api.client'

/**
 * Re-export commonly used OpenAPI generated types for easier access
 */

// Core entity types
export type Client = Schemas.Client
export type User = Schemas.User
export type Role = Schemas.Role
export type Realm = Schemas.Realm
export type Webhook = Schemas.Webhook

// Response types
export type ClientsResponse = Schemas.ClientsResponse
export type UsersResponse = Schemas.UsersResponse
export type GetRolesResponse = Schemas.GetRolesResponse
export type AuthResponse = Schemas.AuthResponse

// Request/Validator types
export type CreateClientValidator = Schemas.CreateClientValidator
export type CreateUserValidator = Schemas.CreateUserValidator
export type CreateRoleValidator = Schemas.CreateRoleValidator
export type UpdateClientValidator = Schemas.UpdateClientValidator
export type UpdateUserValidator = Schemas.UpdateUserValidator
export type UpdateRoleValidator = Schemas.UpdateRoleValidator

// Authentication types
export type AuthenticateRequest = Schemas.AuthenticateRequest
export type AuthenticateResponse = Schemas.AuthenticateResponse
export type AuthenticationStatus = Schemas.AuthenticationStatus

// Credential types
export type CredentialData = Schemas.CredentialData
export type CredentialOverview = Schemas.CredentialOverview

// Settings types
export type RealmSetting = Schemas.RealmSetting
export type RealmLoginSetting = Schemas.RealmLoginSetting

// Other commonly used types
export type RedirectUri = Schemas.RedirectUri
export type JwtToken = Schemas.JwtToken
export type RequiredAction = Schemas.RequiredAction
export type GrantType = Schemas.GrantType
export type WebhookTrigger = Schemas.WebhookTrigger
