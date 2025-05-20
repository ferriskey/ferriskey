import { Role } from "@/api/api.interface";
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color';
import { ColumnDef } from "@/components/ui/data-table";

export const columns: ColumnDef<Role>[] = [
  {
    id: "name",
    header: "Role",
    cell: (role) => (
      <div className="flex items-center gap-3">
        <div className="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center">
          <span className="text-xs font-medium text-primary">{role.name[0].toUpperCase()}</span>
        </div>
        <div>
          <div className="font-medium">{role.name}</div>
          <div className="text-xs text-muted-foreground">{role.name}</div>
        </div>
      </div>
    )
  },
  {
    id: "permissions",
    header: "Permissions",
    cell: (role) => (
      <div className="flex items-center gap-1">
        <BadgeColor color={BadgeColorScheme.PRIMARY}>{role.permissions}</BadgeColor>
      </div>
    )
  },
]