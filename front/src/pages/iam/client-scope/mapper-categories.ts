import type { PillTone } from '@/components/kit'

export interface MapperCategory {
  labelKey: string
  tone: PillTone
}

export function mapperCategory(mapperType: string): MapperCategory {
  const type = mapperType.toLowerCase()

  if (type.includes('role')) return { labelKey: 'mapper_category.role', tone: 'violet' }
  if (type.includes('audience')) return { labelKey: 'mapper_category.audience', tone: 'success' }
  if (type.includes('hardcoded')) return { labelKey: 'mapper_category.hardcoded', tone: 'amber' }
  if (type.includes('attribute')) return { labelKey: 'mapper_category.attribute', tone: 'info' }
  if (type.includes('organization')) {
    return { labelKey: 'mapper_category.organization', tone: 'neutral' }
  }
  if (type.includes('property') || type.includes('full-name') || type.includes('usermodel')) {
    return { labelKey: 'mapper_category.identity', tone: 'info' }
  }

  return { labelKey: 'mapper_category.custom', tone: 'neutral' }
}

export const isRoleMapper = (mapperType: string) => mapperType.toLowerCase().includes('role')

export const isIdentityMapper = (mapperType: string) => {
  const type = mapperType.toLowerCase()
  return (
    type.includes('usermodel') ||
    type.includes('attribute') ||
    type.includes('property') ||
    type.includes('full-name')
  )
}
