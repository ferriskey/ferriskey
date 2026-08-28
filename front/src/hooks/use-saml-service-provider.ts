import type { Schemas } from '@/api/api.client'
import {
  useCreateSamlAttributeMapper,
  useDeleteSamlAttributeMapper,
  useGetSamlAttributeMappers,
  useGetSamlConfig,
  useUpsertSamlConfig,
} from '@/api/saml.api'
import { apiErrorMessage } from '@/lib/api-error'
import { AttributeMapperDraft, DEFAULT_ATTRIBUTE_NAME_FORMAT } from '@/lib/saml'
import { toast } from 'sonner'

export interface SamlAttributeMapperInput extends AttributeMapperDraft {
  nameFormat?: string
}

export function useSamlServiceProvider(realmName?: string, clientId?: string) {
  const { data: config, isLoading: isLoadingConfig } = useGetSamlConfig({ realmName, clientId })
  const { data: mappers = [], isLoading: isLoadingMappers } = useGetSamlAttributeMappers({
    realmName,
    clientId,
  })

  const { mutateAsync: upsertConfig, isPending: isSavingConfig } = useUpsertSamlConfig()
  const { mutateAsync: createMapper, isPending: isCreatingMapper } = useCreateSamlAttributeMapper()
  const { mutateAsync: removeMapper, isPending: isDeletingMapper } = useDeleteSamlAttributeMapper()

  const isConfigured = config != null

  const saveConfig = async (
    payload: Schemas.SetClientSamlConfigValidator,
  ): Promise<boolean> => {
    if (!realmName || !clientId) return false

    try {
      await upsertConfig({ realmName, clientId, payload })
      toast.success(
        isConfigured ? 'Service provider updated' : 'Service provider configured',
      )
      return true
    } catch (error) {
      toast.error(apiErrorMessage(error, 'Could not save the service provider'))
      return false
    }
  }

  const addAttributeMapper = async (input: SamlAttributeMapperInput): Promise<boolean> => {
    if (!realmName || !clientId) return false

    try {
      await createMapper({
        realmName,
        clientId,
        payload: {
          name: input.name.trim(),
          source: input.source,
          name_format: input.nameFormat ?? DEFAULT_ATTRIBUTE_NAME_FORMAT,
        },
      })
      toast.success(`Attribute ${input.name.trim()} mapped`)
      return true
    } catch (error) {
      toast.error(apiErrorMessage(error, 'Could not map the attribute'))
      return false
    }
  }

  const deleteAttributeMapper = async (mapperId: string): Promise<boolean> => {
    if (!realmName || !clientId) return false

    try {
      await removeMapper({ realmName, clientId, mapperId })
      toast.success('Attribute mapping removed')
      return true
    } catch (error) {
      toast.error(apiErrorMessage(error, 'Could not remove the attribute mapping'))
      return false
    }
  }

  const addCommonProfileMappers = async (drafts: AttributeMapperDraft[]): Promise<void> => {
    for (const draft of drafts) {
      await addAttributeMapper(draft)
    }
  }

  return {
    config: config ?? null,
    mappers,
    isConfigured,
    isLoading: isLoadingConfig || isLoadingMappers,
    isSavingConfig,
    isCreatingMapper,
    isDeletingMapper,
    saveConfig,
    addAttributeMapper,
    deleteAttributeMapper,
    addCommonProfileMappers,
  }
}
