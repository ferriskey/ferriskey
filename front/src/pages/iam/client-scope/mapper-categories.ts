import type { PillTone } from '@/components/kit'

export interface MapperCategory {
  label: string
  tone: PillTone
}

export function mapperCategory(mapperType: string): MapperCategory {
  const type = mapperType.toLowerCase()

  if (type.includes('role')) return { label: 'role', tone: 'violet' }
  if (type.includes('audience')) return { label: 'audience', tone: 'success' }
  if (type.includes('hardcoded')) return { label: 'hardcoded', tone: 'amber' }
  if (type.includes('attribute')) return { label: 'attribute', tone: 'info' }
  if (type.includes('organization')) return { label: 'organization', tone: 'neutral' }
  if (type.includes('property') || type.includes('full-name') || type.includes('usermodel')) {
    return { label: 'identity', tone: 'info' }
  }

  return { label: 'custom', tone: 'neutral' }
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
