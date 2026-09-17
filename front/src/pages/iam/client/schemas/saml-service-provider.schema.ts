import { CUSTOM_ATTRIBUTE_SOURCE } from '@/lib/saml'
import { translate } from '@/lib/i18n'
import { z } from 'zod'

export const samlServiceProviderSchema = z.object({
  spEntityId: z
    .string()
    .min(1, { error: () => translate('client:validation.sp_entity_id_required') }),
  acsUrl: z.string().min(1, { error: () => translate('client:validation.acs_url_required') }),
  nameIdFormat: z
    .string()
    .min(1, { error: () => translate('client:validation.name_id_format_required') }),
  signAssertions: z.boolean(),
  signDocuments: z.boolean(),
  wantAuthnRequestsSigned: z.boolean(),
})

export type SamlServiceProviderSchema = z.infer<typeof samlServiceProviderSchema>

export const samlAttributeMapperSchema = z
  .object({
    name: z.string().min(1, { error: () => translate('client:validation.mapper_name_required') }),
    source: z
      .string()
      .min(1, { error: () => translate('client:validation.mapper_source_required') }),
    customKey: z.string(),
    nameFormat: z
      .string()
      .min(1, { error: () => translate('client:validation.mapper_name_format_required') }),
  })
  .refine(
    (values) =>
      values.source !== CUSTOM_ATTRIBUTE_SOURCE || values.customKey.trim().length > 0,
    {
      error: () => translate('client:validation.mapper_custom_key_required'),
      path: ['customKey'],
    }
  )

export type SamlAttributeMapperSchema = z.infer<typeof samlAttributeMapperSchema>
