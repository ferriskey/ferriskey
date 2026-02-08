import { useState } from 'react'
import BlockContent from '@/components/ui/block-content'
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

  return (<div className='w-full'>
    <BlockContent title='General settings'>
      <div className='flex flex-col gap-3'>
        <FormField
          control={form.control}
          name='name'
          render={({ field }) => <InputText label='Realm Name' disabled {...field} />}
        />

        <FormField
          control={form.control}
          name='default_signing_algorithm'
          render={({ field }) => (
            <div>
              <Label>Default Signing Algorithm</Label>
              <Select
                onValueChange={(value) => field.onChange(value)}
                value={field.value}
                disabled
              >
                <SelectTrigger className='w-1/3'>
                  <SelectValue>{field.value}</SelectValue>
                </SelectTrigger>
                <SelectContent position='popper'>
                  {
                    Object.values(SigningAlgorithm).map((value) => {
                      return (
                        <SelectItem value={value}>{value.toString()}</SelectItem>
                      )
                    })
                  }
                </SelectContent>
              </Select>
            </div>
          )}
        />
      </div>
    </BlockContent>

    <BlockContent title='Danger zone' className='border-destructive'>
      <div className='flex items-center justify-between'>
        <div>
          <p className='text-sm font-medium'>Delete this realm</p>
          <p className='text-sm text-muted-foreground'>
            Once deleted, all data associated with this realm will be permanently removed.
          </p>
        </div>
        <Button
          variant='destructive'
          disabled={isMaster}
          onClick={() => setShowDeleteDialog(true)}
        >
          Delete realm
        </Button>
      </div>
      {isMaster && (
        <p className='text-sm text-muted-foreground mt-2'>
          The master realm cannot be deleted.
        </p>
      )}
    </BlockContent>

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
      title={'Save changes'}
      actions={[
        {
          label: 'Save',
          variant: 'default',
          onClick: () => { }
        },
      ]}
      description="You have unsaved changes. Click 'Save' to apply them."
      onCancel={() => {
        form.reset()
      }}
    />
  </div>)
}
