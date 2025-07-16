import { Permissions, RequiredAction, User } from "@/api/api.interface";
import { BadgeColorScheme } from "@/components/ui/badge-color";

export function getBadgeColorFromPermissionVariant (permission: Permissions): BadgeColorScheme {
  if (permission.toString().startsWith('manage')) return BadgeColorScheme.RED
  if (permission.toString().startsWith('create')) return BadgeColorScheme.GREEN
  if (permission.toString().startsWith('view')) return BadgeColorScheme.BLUE
  if (permission.toString().startsWith('query')) return BadgeColorScheme.YELLOW
  return BadgeColorScheme.GRAY
}


export function formatSnakeCaseToTitleCase(snakeCase: string): string {
  return snakeCase
    .replace(/_/g, ' ')
    .toLowerCase()
    .replace(/\b\w/g, (letter) => letter.toUpperCase())
}

export function formatPermissionName(permission: string): string {
  return permission
    .replace(/_/g, ' ')
    .replace(/\b\w/g, l => l.toUpperCase())
}

export function formatRequiredAction(action: RequiredAction): string {
  return formatSnakeCaseToTitleCase(action)
}

 
export function deepEqual<T>(a: T, b: T): boolean {
  if (a === b) return true

  if (a == null || b == null) return a === b

  if (typeof a !== typeof b) return false

  if (a instanceof Date && b instanceof Date) {
    return a.getTime() === b.getTime()
  }

  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false
    return a.every((item, index) => deepEqual(item, b[index]))
  }

  if (typeof a === 'object' && typeof b === 'object' && !Array.isArray(a) && !Array.isArray(b)) {
    const keysA = Object.keys(a as Record<string, unknown>)
    const keysB = Object.keys(b as Record<string, unknown>)
    
    if (keysA.length !== keysB.length) return false
    
    return keysA.every(key => 
      deepEqual(
        (a as Record<string, unknown>)[key], 
        (b as Record<string, unknown>)[key]
      )
    )
  }
  
  return false
}

export function isServiceAccount(user: User): boolean {
  return !!user.client_id
}