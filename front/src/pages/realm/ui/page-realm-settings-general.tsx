import { useState } from 'react'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { UpdateRealmSchema } from '../validators'
import { useFormContext } from 'react-hook-form'
import { SigningAlgorithm } from '@/api/core.interface'
import { Select, SelectItem, SelectTrigger, SelectValue, SelectContent } from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import FloatingActionBar from '@/components/ui/floating-action-bar'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

export interface PageRealmSettingsGeneralProps {
  hasChanges: boolean
  realmName: string
  isMaster: boolean
  onDeleteRealm: () => void
}

export default function PageRealmSettingsGeneral({ hasChanges, realmName, isMaster, onDeleteRealm }: PageRealmSettingsGeneralProps) {
  const form = useFormContext<UpdateRealmSchema>()
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)

  return (
    <div className='flex flex-col gap-8'>
      {/* General settings */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-muted-foreground mb-0.5'>Realm configuration</p>
          <h2 className='text-base font-semibold'>General Settings</h2>
        </div>

        <FormField
          control={form.control}
          name='name'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Realm Name</p>
                <p className='text-sm text-muted-foreground mt-0.5'>The unique name identifying this realm.</p>
              </div>
              <div className='w-1/2'>
                <InputText label='Realm Name' disabled {...field} />
              </div>
            </div>
          )}
        />

        <FormField
          control={form.control}
          name='default_signing_algorithm'
          render={({ field }) => (
            <div className='flex items-start justify-between py-4 border-t'>
              <div className='w-1/3'>
                <p className='text-sm font-medium'>Default Signing Algorithm</p>
                <p className='text-sm text-muted-foreground mt-0.5'>Algorithm used to sign tokens for this realm.</p>
              </div>
              <div className='w-1/2'>
                <Label className='text-sm text-muted-foreground mb-1.5 block'>Algorithm</Label>
                <Select onValueChange={field.onChange} value={field.value} disabled>
                  <SelectTrigger className='w-48'>
                    <SelectValue>{field.value}</SelectValue>
                  </SelectTrigger>
                  <SelectContent position='popper'>
                    {Object.values(SigningAlgorithm).map((value) => (
                      <SelectItem key={value} value={value}>{value.toString()}</SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}
        />
      </div>

      {/* Danger zone */}
      <div className='flex flex-col gap-1'>
        <div className='mb-4'>
          <p className='text-xs text-destructive/70 mb-0.5'>Irreversible actions</p>
          <h2 className='text-base font-semibold text-destructive'>Danger Zone</h2>
        </div>

        <div className='flex items-center justify-between py-4 border-t border-destructive/20'>
          <div className='w-2/3'>
            <p className='text-sm font-medium'>Delete this realm</p>
            <p className='text-sm text-muted-foreground mt-0.5'>
              Once deleted, all data associated with this realm will be permanently removed.
              {isMaster && ' The master realm cannot be deleted.'}
            </p>
          </div>
          <Button variant='destructive' disabled={isMaster} onClick={() => setShowDeleteDialog(true)}>
            Delete realm
          </Button>
        </div>
      </div>

      <ConfirmDeleteAlert
        open={showDeleteDialog}
        title='Delete realm'
        description={`This will permanently delete the realm "${realmName}" and all its data including users, clients, and roles.`}
        confirmText={realmName}
        onConfirm={onDeleteRealm}
        onCancel={() => setShowDeleteDialog(false)}
      />

      <FloatingActionBar
        show={hasChanges}
        title='Save Changes'
        actions={[{ label: 'Save', variant: 'default', onClick: () => { } }]}
        description="You have unsaved changes. Click 'Save' to apply them."
        onCancel={form.reset}
      />
    </div>
  )
}
