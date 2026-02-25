import { useCreateRedirectUri, useDeleteRedirectUri } from '@/api/redirect_uris.api'
import { Button } from '@/components/ui/button'
import { InputText } from '@/components/ui/input-text'
import { RouterParams } from '@/routes/router'
import { Trash2 } from 'lucide-react'
import { useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { toast } from 'sonner'
import { Schemas } from '@/api/api.client.ts'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { z } from 'zod'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { Form, FormField } from '@/components/ui/form'
import RedirectUri = Schemas.RedirectUri

export interface ManageRedirectUrisProps {
  redirectUris: RedirectUri[]
  refetch: () => void
}

const createRedirectUriSchema = z.object({
  newRedirectUri: z.string().min(1, { message: 'Redirect URI is required' }),
})

type CreateRedirectUriSchema = z.infer<typeof createRedirectUriSchema>

export default function ManageRedirectUris({ redirectUris, refetch }: ManageRedirectUrisProps) {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const { mutateAsync: deleteRedirectUri, isSuccess: deleteRedirectUriSuccess } =
    useDeleteRedirectUri()
  const { mutateAsync: createRedirectUri, isSuccess: createRedirectUriSuccess } =
    useCreateRedirectUri()

  const form = useForm<CreateRedirectUriSchema>({
    resolver: zodResolver(createRedirectUriSchema),
    defaultValues: {
      newRedirectUri: '',
    },
  })

  const handleDeleteRedirectUri = async (uriId: string) => {
    if (!realm_name || !client_id) return

    ask({
      title: 'Delete redirect URI?',
      description: 'Are you sure you want to delete this redirect URI?',
      onConfirm: async () => {
        await deleteRedirectUri({
          realmName: realm_name,
          clientId: client_id,
          redirectUriId: uriId,
        })
        refetch()
        close()
      },
    })
  }

  const onSubmit = async (values: CreateRedirectUriSchema) => {
    if (!realm_name || !client_id) return

    await createRedirectUri({
      realmName: realm_name,
      clientId: client_id,
      payload: { value: values.newRedirectUri },
    })

    refetch()
    form.reset()
  }

  useEffect(() => {
    if (createRedirectUriSuccess) {
      toast.success('Redirect URI added successfully')
    }
  }, [createRedirectUriSuccess])

  useEffect(() => {
    if (deleteRedirectUriSuccess) {
      toast.success('Redirect URI deleted successfully')
    }
  }, [deleteRedirectUriSuccess])

  return (
    <>
      <div className='flex flex-col gap-4'>
        {redirectUris.map((uri, index) => (
          <div key={index} className='flex gap-2 items-center'>
            <InputText
              name='redirect_uri'
              label={`Redirect URI ${index + 1}`}
              value={uri.value}
              disabled
              className='flex-grow'
            />

            <div>
              <Button
                className='text-red-500'
                variant='ghost'
                size='icon'
                onClick={() => {
                  handleDeleteRedirectUri(uri.id)
                }}
              >
                <Trash2 size={14} />
              </Button>
            </div>
          </div>
        ))}

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className='flex flex-col gap-2'>
            <FormField
              control={form.control}
              name='newRedirectUri'
              render={({ field }) => (
                <InputText
                  {...field}
                  label='Add new Redirect URI'
                  className='flex-grow'
                  error={form.formState.errors?.newRedirectUri?.message}
                />
              )}
            />

            <Button type='submit'>Add Redirect URI</Button>
          </form>
        </Form>
      </div>

      <ConfirmDeleteAlert
        title={confirm.title}
        description={confirm.description}
        open={confirm.open}
        onConfirm={confirm.onConfirm}
        onCancel={close}
      />
    </>
  )
}
