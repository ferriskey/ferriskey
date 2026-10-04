const PARENT_KEY = 'parent_group_id'
const ROOT_KEY = 'is_root'
const ROOT_ONLY = 'true'

export function walkDownPatch(parentId: string): Record<string, string> {
  return { [PARENT_KEY]: parentId, [ROOT_KEY]: '' }
}

export function groupFilterPatch(key: string, value: string): Record<string, string> {
  if (key === PARENT_KEY && value) return walkDownPatch(value)
  if (key === ROOT_KEY && value === ROOT_ONLY) return { [ROOT_KEY]: ROOT_ONLY, [PARENT_KEY]: '' }
  return { [key]: value }
}
