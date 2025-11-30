import { DataTable } from '@/components/ui/data-table'
import { columnsUserCredential } from '../columns/list-user-credential.column'
import SetPasswordFeature from '../feature/modals/set-password-feature'
import { Trash2 } from 'lucide-react'
import { Schemas } from '@/api/api.client'
import { useConfirmDeleteAlert } from '@/hooks/use-confirm-delete-alert.ts'
import { ConfirmDeleteAlert } from '@/components/confirm-delete-alert'

import CredentialOverview = Schemas.CredentialOverview

export interface PageCredentialsProps {
  credentials: CredentialOverview[]
  handleDeleteUserCredential: (credentialId: string) => void
}

export default function PageCredentials({
  credentials,
  handleDeleteUserCredential,
}: PageCredentialsProps) {
  const { confirm, ask, close } = useConfirmDeleteAlert()

  const onRowDelete = (credential: CredentialOverview) => {
    ask({
      title: 'Delete credential?',
      description: `Are you sure you want to delete "${credential.credential_type}"?`,
      onConfirm: () => {
        handleDeleteUserCredential(credential.id)
        close()
      },
    })
  }
  return (
    <>
      <DataTable
        data={credentials}
        columns={columnsUserCredential}
        searchPlaceholder='Search a credential...'
        enableSelection={true}
        emptyState={<EmptyCredential />}
        onDeleteSelected={() => {
          credentials.forEach((c) => {
            handleDeleteUserCredential(c.id)
          })
        }}
        rowActions={[
          {
            label: 'Delete',
            icon: <Trash2 className='w-4 h-4' />,
            variant: 'destructive',
            onClick: (credential) => {
              onRowDelete(credential)
            },
          },
        ]}
      />
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

function EmptyCredential() {
  return (
    <div className='text-center flex flex-col gap-3 py-8'>
      <img src='/event-placeholder-light.svg' alt='No credentials' className='mx-auto mb-4 w-40' />
      <div className=''>
        <h2 className='text-lg font-semibold text-neutral-600'>No Credentials Found</h2>
        <p className='text-muted-foreground'>You have no credentials available.</p>
      </div>
      <div>
        <SetPasswordFeature />
      </div>
    </div>
  )
}
