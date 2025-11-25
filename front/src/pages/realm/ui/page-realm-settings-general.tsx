import BlockContent from '@/components/ui/block-content'
import { FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { UpdateRealmSchema } from '../validators'
import { useFormContext } from 'react-hook-form'
import { SigningAlgorithm } from '@/api/core.interface'
import { Select, SelectItem, SelectTrigger, SelectValue, SelectContent } from '@/components/ui/select'
import { Label } from '@/components/ui/label'
import FloatingActionBar from '@/components/ui/floating-action-bar'

export interface PageRealmSettingsGeneralProps {
  hasChanges: boolean
}


export default function PageRealmSettingsGeneral({ hasChanges }: PageRealmSettingsGeneralProps) {
  const form = useFormContext<UpdateRealmSchema>()

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
