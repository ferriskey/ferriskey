import { Role } from '@/api/api.interface'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Skeleton } from '@/components/ui/skeleton'
import { ArrowLeft } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useNavigate } from 'react-router-dom'
import BlockContent from '@/components/ui/block-content.tsx'
import { FormField } from '@/components/ui/form.tsx'
import { UseFormReturn } from 'react-hook-form'
import { UpdateRoleSchema } from '@/pages/role/schemas/update-role.schema.ts'
import { InputText } from '@/components/ui/input-text.tsx'
import BadgeColor, { BadgeColorScheme } from '@/components/ui/badge-color.tsx'

export interface PageRoleSettingsProps {
  role?: Role
  form: UseFormReturn<UpdateRoleSchema>
  isLoading?: boolean
  realmName: string
}

export default function PageRoleSettings({ role, isLoading, realmName, form }: PageRoleSettingsProps) {
  const navigate = useNavigate();

  const handleBackClick = () => {
    navigate(`/realms/${realmName}/roles`);
  };

  if (isLoading) {
    return (
      <div className="space-y-6 p-6">
        <div className="flex items-center gap-4">
          <Skeleton className="h-10 w-10" />
          <div className="space-y-2">
            <Skeleton className="h-8 w-48" />
            <Skeleton className="h-4 w-64" />
          </div>
        </div>

        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Skeleton className="h-4 w-16" />
              <Skeleton className="h-6 w-40" />
            </div>
            <div className="space-y-2">
              <Skeleton className="h-4 w-20" />
              <Skeleton className="h-6 w-24" />
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!role) {
    return (
      <div className="space-y-6 p-6">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={handleBackClick}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <div>
            <h1 className="text-2xl font-bold tracking-tight">Rôle introuvable</h1>
            <p className="text-muted-foreground">
              Le rôle demandé n'existe pas dans le realm {realmName}
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="">
      <div>
        <BlockContent title={"Role informations"}>
          <div className="flex flex-col gap-3">
            <FormField
              control={form.control}
              name={"name"}
              render={({ field }) => (
                <InputText
                  name={field.name}
                  label={"Name"}
                  {...field}
                />
              )}
            />

            <FormField
              control={form.control}
              name={"description"}
              render={({ field }) => (
                <InputText
                  name={field.name}
                  label={"Description"}
                  {...field}
                />
              )}
            />

          </div>
        </BlockContent>

      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-6">

        <div className="border p-4 rounded-sm flex flex-col">
          <span className="text-xs text-muted-foreground">
            Number of permissions
          </span>

          <div>
            <BadgeColor color={BadgeColorScheme.BLUE}>
              {role.permissions.length}
            </BadgeColor>
          </div>



        </div>

      </div>

      <Card>
        <CardHeader>
          <CardTitle>Informations générales</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Nom du rôle</label>
              <p className="text-lg font-medium">{role.name}</p>
            </div>

            {role.description && (
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground">Description</label>
                <p className="text-lg">{role.description}</p>
              </div>
            )}

            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Permissions</label>
              <Badge variant="secondary">{role.permissions}</Badge>
            </div>

            {role.client && (
              <div className="space-y-2">
                <label className="text-sm font-medium text-muted-foreground">Client associé</label>
                <p className="text-lg">{role.client.name}</p>
              </div>
            )}

            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Date de création</label>
              <p className="text-lg">{new Date(role.created_at).toLocaleDateString('fr-FR')}</p>
            </div>

            <div className="space-y-2">
              <label className="text-sm font-medium text-muted-foreground">Dernière modification</label>
              <p className="text-lg">{new Date(role.updated_at).toLocaleDateString('fr-FR')}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
