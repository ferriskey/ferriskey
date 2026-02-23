import {
  useCreatePostLogoutRedirectUri,
  useDeletePostLogoutRedirectUri,
  useGetPostLogoutRedirectUris,
} from '@/api/post_logout_redirect_uris.api'
import { Button } from '@/components/ui/button'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'
import { Form, FormField } from '@/components/ui/form'
import { InputText } from '@/components/ui/input-text'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { RouterParams } from '@/routes/router'
import { zodResolver } from '@hookform/resolvers/zod'
import { Trash2 } from 'lucide-react'
import { useForm } from 'react-hook-form'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { z } from 'zod'

const createPostLogoutRedirectUriSchema = z.object({
  newPostLogoutRedirectUri: z.string().min(1, { message: 'Post-logout redirect URI is required' }),
})

type CreatePostLogoutRedirectUriSchema = z.infer<typeof createPostLogoutRedirectUriSchema>

export default function ManagePostLogoutRedirectUris() {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const { data: postLogoutRedirectUris = [], refetch } = useGetPostLogoutRedirectUris({
    realmName: realm_name,
    clientId: client_id,
  })
  const { mutateAsync: deletePostLogoutRedirectUri } = useDeletePostLogoutRedirectUri()
  const { mutateAsync: createPostLogoutRedirectUri } = useCreatePostLogoutRedirectUri()

  const form = useForm<CreatePostLogoutRedirectUriSchema>({
    resolver: zodResolver(createPostLogoutRedirectUriSchema),
    defaultValues: {
      newPostLogoutRedirectUri: '',
    },
  })

  const handleDeletePostLogoutRedirectUri = async (uriId: string) => {
    if (!realm_name || !client_id) return

    ask({
      title: 'Delete post-logout redirect URI?',
      description: 'Are you sure you want to delete this post-logout redirect URI?',
      onConfirm: async () => {
        try {
          await deletePostLogoutRedirectUri({
            realmName: realm_name,
            clientId: client_id,
            redirectUriId: uriId,
          })

          await refetch()
          toast.success('Post-logout redirect URI deleted successfully')
          close()
        } catch (error) {
          console.error('Failed to delete post-logout redirect URI:', error)
          toast.error('Failed to delete post-logout redirect URI')
        }
      },
    })
  }

  const onSubmit = async (values: CreatePostLogoutRedirectUriSchema) => {
    if (!realm_name || !client_id) return

    try {
      await createPostLogoutRedirectUri({
        realmName: realm_name,
        clientId: client_id,
        payload: { value: values.newPostLogoutRedirectUri },
      })

      await refetch()
      toast.success('Post-logout redirect URI added successfully')
      form.reset()
    } catch (error) {
      console.error('Failed to create post-logout redirect URI:', error)
      toast.error('Failed to create post-logout redirect URI')
    }
  }

  return (
    <>
      <div className='flex flex-col gap-4'>
        {postLogoutRedirectUris.map((uri, index) => (
          <div key={uri.id} className='flex gap-2 items-center'>
            <InputText
              name='post_logout_redirect_uri'
              label={`Post-logout Redirect URI ${index + 1}`}
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
                  handleDeletePostLogoutRedirectUri(uri.id)
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
              name='newPostLogoutRedirectUri'
              render={({ field }) => (
                <InputText
                  {...field}
                  label='Add new Post-logout Redirect URI'
                  className='flex-grow'
                  error={form.formState.errors?.newPostLogoutRedirectUri?.message}
                />
              )}
            />

            <Button type='submit'>Add Post-logout Redirect URI</Button>
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
