import { useCreateWebOrigin, useDeleteWebOrigin, useGetWebOrigins } from '@/api/web_origins.api'
import { Button } from '@/components/ui/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Form, FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { DERIVED_ORIGIN_SENTINEL, isWebOriginValue } from '@/lib/web-origin'
import { RouterParams } from '@/routes/router'
import { zodResolver } from '@hookform/resolvers/zod'
import { Trash2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { z } from 'zod'

const createWebOriginSchema = z.object({
  newWebOrigin: z
    .string()
    .min(1, { message: 'Web origin is required' })
    .refine(isWebOriginValue, {
      message: `Enter an origin such as https://app.example.com — no path, no wildcard — or ${DERIVED_ORIGIN_SENTINEL} to derive them from this client's redirect URIs`,
    }),
})

type CreateWebOriginSchema = z.infer<typeof createWebOriginSchema>

export default function ManageWebOrigins() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const { data: webOrigins = [], refetch } = useGetWebOrigins({
    realmName: realm_name,
    clientId: client_id,
  })
  const { mutateAsync: deleteWebOrigin } = useDeleteWebOrigin()
  const { mutateAsync: createWebOrigin } = useCreateWebOrigin()

  const form = useForm<CreateWebOriginSchema>({
    resolver: zodResolver(createWebOriginSchema),
    defaultValues: {
      newWebOrigin: '',
    },
  })

  const handleDeleteWebOrigin = async (webOriginId: string) => {
    if (!realm_name || !client_id) return

    ask({
      title: 'Delete web origin?',
      description:
        'Browsers may keep a cached preflight for a few minutes, and other API replicas for up to 30 seconds.',
      onConfirm: async () => {
        try {
          await deleteWebOrigin({
            realmName: realm_name,
            clientId: client_id,
            webOriginId,
          })

          await refetch()
          toast.success('Web origin deleted successfully')
          close()
        } catch (error) {
          toast.error(error instanceof Error ? error.message : 'Failed to delete web origin')
        }
      },
    })
  }

  const onSubmit = async (values: CreateWebOriginSchema) => {
    if (!realm_name || !client_id) return

    try {
      await createWebOrigin({
        realmName: realm_name,
        clientId: client_id,
        payload: { value: values.newWebOrigin.trim() },
      })

      await refetch()
      toast.success('Web origin added successfully')
      form.reset()
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'Failed to create web origin')
    }
  }

  return (
    <>
      <div className='flex flex-col gap-4'>
        {webOrigins.map((origin, index) => (
          <div key={origin.id} className='flex gap-2 items-center'>
            <InputText
              name='web_origin'
              label={
                origin.value === DERIVED_ORIGIN_SENTINEL
                  ? 'Derived from redirect URIs'
                  : `Web Origin ${index + 1}`
              }
              value={origin.value}
              disabled
              className='flex-grow'
            />

            <div>
              <Button
                className='text-red-500'
                variant='ghost'
                size='icon'
                aria-label={`Remove web origin ${origin.value}`}
                onClick={() => {
                  handleDeleteWebOrigin(origin.id)
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
              name='newWebOrigin'
              render={({ field }) => (
                <InputText
                  {...field}
                  label='Add new Web Origin'
                  className='flex-grow'
                  error={form.formState.errors?.newWebOrigin?.message}
                />
              )}
            />

            <Button type='submit'>Add Web Origin</Button>
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
