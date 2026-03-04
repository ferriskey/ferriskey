import { Schemas } from '@/api/api.client'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { UseFormReturn } from 'react-hook-form'
import { UpdateProtocolMapperSchema } from '../../schemas/update-protocol-mapper.schema'

import ProtocolMapper = Schemas.ProtocolMapper

interface EditProtocolMapperModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mapper: ProtocolMapper | null
  form: UseFormReturn<UpdateProtocolMapperSchema>
  isValid: boolean
  isPending: boolean
  handleSubmit: () => void
}

export default function EditProtocolMapperModal({
  open,
  onOpenChange,
  mapper,
  form,
  isValid,
  isPending,
  handleSubmit,
}: EditProtocolMapperModalProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Edit Protocol Mapper</DialogTitle>
        </DialogHeader>
        <DialogBody>
          <form onSubmit={handleSubmit} className='flex flex-col gap-4'>
            <FormField
              control={form.control}
              name='name'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Name</FormLabel>
                  <FormControl>
                    <Input placeholder={mapper?.name ?? 'Mapper name'} {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name='mapper_type'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Mapper Type</FormLabel>
                  <FormControl>
                    <Input placeholder={mapper?.mapper_type ?? 'Mapper type'} {...field} />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name='config'
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Config (JSON)</FormLabel>
                  <FormControl>
                    <textarea
                      placeholder='{"key": "value"}'
                      rows={5}
                      className='flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono shadow-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 resize-none'
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
          </form>
        </DialogBody>
        <DialogFooter>
          <Button variant='outline' type='button' onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button type='button' onClick={handleSubmit} disabled={!isValid || isPending}>
            {isPending ? 'Saving...' : 'Save'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
