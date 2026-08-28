import { CUSTOM_ATTRIBUTE_SOURCE } from '@/lib/saml'
import { z } from 'zod'

export const samlServiceProviderSchema = z.object({
  spEntityId: z.string().min(1, { message: 'Entity ID is required' }),
  acsUrl: z.string().min(1, { message: 'Assertion Consumer Service URL is required' }),
  nameIdFormat: z.string().min(1, { message: 'Name ID format is required' }),
  signAssertions: z.boolean(),
  signDocuments: z.boolean(),
  wantAuthnRequestsSigned: z.boolean(),
})

export type SamlServiceProviderSchema = z.infer<typeof samlServiceProviderSchema>

export const samlAttributeMapperSchema = z
  .object({
    name: z.string().min(1, { message: 'Attribute name is required' }),
    source: z.string().min(1, { message: 'Pick which user detail to send' }),
    customKey: z.string(),
    nameFormat: z.string().min(1, { message: 'Name format is required' }),
  })
  .refine(
    (values) =>
      values.source !== CUSTOM_ATTRIBUTE_SOURCE || values.customKey.trim().length > 0,
    { message: 'Attribute key is required', path: ['customKey'] }
  )

export type SamlAttributeMapperSchema = z.infer<typeof samlAttributeMapperSchema>
