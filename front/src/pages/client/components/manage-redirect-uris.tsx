import { useCreateRedirectUri, useDeleteRedirectUri } from '@/api/redirect_uris.api'
import { Button } from '@/components/ui/button'
import { InputText } from '@/components/ui/input-text'
import { RouterParams } from '@/routes/router'
import { Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useParams } from 'react-router'
import { toast } from 'sonner'
import { Schemas } from '@/api/api.client.ts'
import RedirectUri = Schemas.RedirectUri

export interface ManageRedirectUrisProps {
  redirectUris: RedirectUri[]
}

export default function ManageRedirectUris({ redirectUris }: ManageRedirectUrisProps) {
  const { realm_name, client_id } = useParams<RouterParams>()
  const { mutate: deleteRedirectUri, isSuccess: deleteRedirectUriSuccess } = useDeleteRedirectUri()
  const { mutate: createRedirectUri, isSuccess: createRedirectUriSuccess } = useCreateRedirectUri()
  const [newRedirectUri, setNewRedirectUri] = useState<string>('')

  const handleDeleteRedirectUri = (uriId: string) => {
    if (!realm_name || !client_id) return

    deleteRedirectUri({
      realmName: realm_name,
      clientId: client_id,
      redirectUriId: uriId,
    })
  }

  const handleAddRedirectUri = () => {
    if (!realm_name || !client_id || !newRedirectUri) return

    createRedirectUri({
      realmName: realm_name,
      clientId: client_id,
      payload: { value: newRedirectUri },
    })

    setNewRedirectUri('')
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
    <div className="flex flex-col gap-4">
      {redirectUris.map((uri, index) => (
        <div key={index} className="flex gap-2 items-center">
          <InputText
            name="redirect_uri"
            label={`Redirect URI ${index + 1}`}
            value={uri.value}
            disabled
            className="flex-grow"
          />

          <div>
            <Button
              className="text-red-500"
              variant="ghost"
              size="icon"
              onClick={() => {
                handleDeleteRedirectUri(uri.id)
              }}
            >
              <Trash2 size={14} />
            </Button>
          </div>
        </div>
      ))}

      <div className="flex flex-col gap-2">
        <InputText
          name="new_redirect_uri"
          label="Add new Redirect URI"
          onChange={(e) => setNewRedirectUri(e as string)}
          value={newRedirectUri}
          className="flex-grow"
        />

        <Button onClick={handleAddRedirectUri} disabled={!newRedirectUri}>
          Add Redirect URI
        </Button>
      </div>
    </div>
  )
}
