import {
  ALL_MAPPER_TEMPLATES,
  type MapperTemplate,
} from '@/pages/client-scope/constants/protocol-mapper-templates'

export const templateById = (templateId: string | null): MapperTemplate | null =>
  ALL_MAPPER_TEMPLATES.find((template) => template.id === templateId) ?? null

export const templateForType = (mapperType: string): MapperTemplate | null =>
  ALL_MAPPER_TEMPLATES.find(
    (template) => template.mapper_type === mapperType && !template.isCustom
  ) ?? null
