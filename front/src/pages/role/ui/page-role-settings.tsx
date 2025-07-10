import { Role } from '@/api/api.interface'
import { Card, CardContent, CardHeader } from '@/components/ui/card'
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
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRoleSettingsProps {
  role?: Role
  form: UseFormReturn<UpdateRoleSchema>
  isLoading?: boolean
  realmName: string
  hasChanges: boolean
  handleSubmit: () => void
}

export default function PageRoleSettings({
  role,
  isLoading,
  realmName,
  form,
  hasChanges,
  handleSubmit,
}: PageRoleSettingsProps) {
  const navigate = useNavigate()

  const handleBackClick = () => {
    navigate(`/realms/${realmName}/roles`)
  }

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
    )
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
    )
  }

  return (
    <div className="">
      <div>
        <BlockContent title={'Role details'}>
          <div className="flex flex-col gap-3">
            <FormField
              control={form.control}
              name={'name'}
              render={({ field }) => <InputText label={'Name'} {...field} />}
            />

            <FormField
              control={form.control}
              name={'description'}
              render={({ field }) => <InputText label={'Description'} {...field} />}
            />
          </div>
        </BlockContent>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-6">
        <div className="border p-4 rounded-sm flex flex-col gap-3">
          <span className="text-xs text-muted-foreground">Number of permissions</span>

          <div>
            <BadgeColor color={BadgeColorScheme.BLUE}>{role.permissions.length}</BadgeColor>
          </div>
        </div>

        <div className="border p-4 rounded-sm flex flex-col gap-3">
          <span className="text-xs text-muted-foreground">Client</span>

          <div>
            <BadgeColor color={BadgeColorScheme.PRIMARY}>{role.client?.client_id}</BadgeColor>
          </div>
        </div>

        <div className="border p-4 rounded-sm flex flex-col gap-3">
          <span className="text-xs text-muted-foreground">Created at </span>

          <div>
            <BadgeColor color={BadgeColorScheme.GREEN}>
              {new Date(role.created_at).toLocaleDateString('fr-FR')}
            </BadgeColor>
          </div>
        </div>
      </div>

      <FloatingActionBar
        show={hasChanges}
        title="Save changes"
        actions={[
          {
            label: 'Save',
            variant: 'default',
            onClick: form.handleSubmit(handleSubmit),
          },
        ]}
        description="You have unsaved changes. Do you want to save them?"
        onCancel={() => form.reset()}
      />
    </div>
  )
}
