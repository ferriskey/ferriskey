import { User } from "@/api/api.interface";
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color';
import { ColumnDef } from "@/components/ui/data-table";
import clsx from 'clsx';

function isServiceAccount(user: User) {
  return !user.firstname && !user.lastname && user.username.startsWith('service-account-');
}

export const columns: ColumnDef<User>[] = [
  {
    id: "name",
    header: "Utilisateur",
    cell: (user) => {
      const isSA = isServiceAccount(user);
      return (
        (
          <div className="flex items-center gap-3">
            <div className="h-8 w-8 rounded-full bg-primary/10 flex items-center justify-center">
              <span className="text-xs font-medium text-primary">{isSA ? 'SA' : user.firstname?.[0]?.toUpperCase() || 'U'}</span>
            </div>
            <div>
              {isSA ? (
                <div className="font-medium">Service Account</div>
              ) : (
                <>
                  <div className="font-medium">{user.firstname} {user.lastname}</div>
                </>
              )}
              <div className="text-xs text-muted-foreground">{user.username}</div>
            </div>
          </div>
        )
      )
    },
  },
  {
    id: "type",
    header: "Type",
    cell: (user) => {
      if (isServiceAccount(user)) {
        return (
          <BadgeColor color={BadgeColorScheme.PRIMARY}>Service Account</BadgeColor>
        )
      }
    },
  },
  {
    id: "roles",
    header: "Rôles",
    cell: (user) => {
      const copy = [...user.roles]
      const roles = copy.splice(0, 1);
      const hiddenCount = copy.length > 0 ? `+${copy.length}` : null;

      return (
        <div className="flex items-center gap-1">
          {roles.map((role) => (
            <BadgeColor key={role.id} color={BadgeColorScheme.PRIMARY}>{role.name}</BadgeColor>
          ))}
          {hiddenCount && (
            <BadgeColor color={BadgeColorScheme.GRAY}>{hiddenCount}</BadgeColor>
          )}
        </div>
      )
    },
  },
  {
    id: "status",
    header: "Statut",
    cell: (user) => (
      <div className="flex items-center gap-2">
        <span
          className={clsx(
            "h-2 w-2 rounded-full animate-pulse",
            user.enabled ? "bg-emerald-500" : "bg-red-500"
          )}
        ></span>
        <span>{user.enabled ? "Activé" : "Désactivé"}</span>
      </div>
    ),
  },
]