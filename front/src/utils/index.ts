import { Permissions } from "@/api/api.interface";
import { BadgeColorScheme } from "@/components/ui/badge-color";

export function getBadgeColorFromPermissionVariant (permission: Permissions): BadgeColorScheme {
  if (permission.toString().startsWith('manage')) return BadgeColorScheme.RED
  if (permission.toString().startsWith('create')) return BadgeColorScheme.GREEN
  if (permission.toString().startsWith('view')) return BadgeColorScheme.BLUE
  if (permission.toString().startsWith('query')) return BadgeColorScheme.YELLOW
  return BadgeColorScheme.GRAY
}

export function formatPermissionName(permission: string): string {
  return permission
    .replace(/_/g, ' ')
    .replace(/\b\w/g, l => l.toUpperCase())
}