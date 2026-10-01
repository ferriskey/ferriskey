import { zodResolver } from '@hookform/resolvers/zod'
import { ArrowLeftRight, Plus, Trash2 } from 'lucide-react'
import { useEffect } from 'react'
import { Controller, useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit/button'
import {
  ChipInput,
  ConfirmDestructiveDialog,
  EmptyState,
  FieldRow,
  Pill,
  Section,
  SwitchField,
} from '@/components/kit'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { tokens } from '@/styles/style-tokens'
import { cn } from '@/lib/utils'
import { Schemas } from '@/api/api.client'
import {
  tokenExchangePolicySchema,
  type TokenExchangePolicySchema,
} from '../schemas/token-exchange-policy.schema'

import Client = Schemas.Client
import TokenExchangePolicy = Schemas.TokenExchangePolicy

export interface ClientTokenExchangeTabProps {
  client: Client
  policies: TokenExchangePolicy[]
  audiences: Client[]
  isLoading: boolean
  isError: boolean
  isCreating: boolean
  isDeleting: boolean
  createOpen: boolean
  pendingDelete: TokenExchangePolicy | null
  onCreateOpenChange: (open: boolean) => void
  onCreate: (values: TokenExchangePolicySchema) => void
  onRequestDelete: (policy: TokenExchangePolicy) => void
  onCancelDelete: () => void
  onConfirmDelete: () => void
}

const EMPTY_FORM: TokenExchangePolicySchema = {
  targetAudience: '',
  allowedScopes: [],
  allowImpersonation: false,
  allowDelegation: false,
}

function CreatePolicyDialog({
  open,
  onOpenChange,
  audiences,
  isCreating,
  onCreate,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  audiences: Client[]
  isCreating: boolean
  onCreate: (values: TokenExchangePolicySchema) => void
}) {
  const { t } = useTranslation('client')

  const {
    control,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<TokenExchangePolicySchema>({
    resolver: zodResolver(tokenExchangePolicySchema),
    defaultValues: EMPTY_FORM,
  })

  useEffect(() => {
    if (!open) reset(EMPTY_FORM)
  }, [open, reset])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className='sm:max-w-lg'>
        <form onSubmit={handleSubmit(onCreate)} className='space-y-1'>
          <DialogHeader>
            <DialogTitle>{t('token_exchange.create.title')}</DialogTitle>
            <DialogDescription>{t('token_exchange.create.description')}</DialogDescription>
          </DialogHeader>

          <FieldRow
            layout='stacked'
            label={t('token_exchange.create.audience.label')}
            description={t('token_exchange.create.audience.description')}
          >
            <Controller
              control={control}
              name='targetAudience'
              render={({ field }) => (
                <Select value={field.value} onValueChange={field.onChange}>
                  <SelectTrigger className='w-full'>
                    <SelectValue placeholder={t('token_exchange.create.audience.placeholder')} />
                  </SelectTrigger>
                  <SelectContent>
                    {audiences.map((candidate) => (
                      <SelectItem key={candidate.id} value={candidate.client_id}>
                        {candidate.client_id}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              )}
            />
            {errors.targetAudience && (
              <p className='mt-1 text-xs text-fk-danger'>{errors.targetAudience.message}</p>
            )}
          </FieldRow>

          <FieldRow
            layout='stacked'
            label={t('token_exchange.create.scopes.label')}
            description={t('token_exchange.create.scopes.description')}
          >
            <Controller
              control={control}
              name='allowedScopes'
              render={({ field }) => (
                <ChipInput
                  values={field.value}
                  onChange={field.onChange}
                  placeholder={t('token_exchange.create.scopes.placeholder')}
                  emptyHint={t('token_exchange.create.scopes.empty')}
                />
              )}
            />
          </FieldRow>

          <FieldRow
            layout='stacked'
            label={t('token_exchange.create.impersonation.label')}
            description={t('token_exchange.create.impersonation.description')}
          >
            <Controller
              control={control}
              name='allowImpersonation'
              render={({ field }) => (
                <SwitchField checked={field.value} onCheckedChange={field.onChange} />
              )}
            />
          </FieldRow>

          <FieldRow
            layout='stacked'
            label={t('token_exchange.create.delegation.label')}
            description={t('token_exchange.create.delegation.description')}
          >
            <Controller
              control={control}
              name='allowDelegation'
              render={({ field }) => (
                <SwitchField checked={field.value} onCheckedChange={field.onChange} />
              )}
            />
          </FieldRow>

          <DialogFooter>
            <Button
              type='button'
              variant='ghost'
              disabled={isCreating}
              onClick={() => onOpenChange(false)}
            >
              {t('token_exchange.create.cancel')}
            </Button>
            <Button type='submit' disabled={isCreating}>
              {isCreating ? t('token_exchange.create.submitting') : t('token_exchange.create.submit')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default function ClientTokenExchangeTab({
  client,
  policies,
  audiences,
  isLoading,
  isError,
  isCreating,
  isDeleting,
  createOpen,
  pendingDelete,
  onCreateOpenChange,
  onCreate,
  onRequestDelete,
  onCancelDelete,
  onConfirmDelete,
}: ClientTokenExchangeTabProps) {
  const { t } = useTranslation('client')

  const addButton = (
    <Button size='sm' disabled={audiences.length === 0} onClick={() => onCreateOpenChange(true)}>
      <Plus /> {t('token_exchange.add')}
    </Button>
  )

  return (
    <>
      {!client.token_exchange_enabled && (
        <p className='rounded-md border border-fk-line bg-neutral-50 px-3 py-2 text-xs text-neutral-600 dark:bg-fk-surface dark:text-neutral-400'>
          {t('token_exchange.disabled_hint')}
        </p>
      )}

      <Section
        title={t('token_exchange.title', { total: policies.length })}
        description={t('token_exchange.description')}
        action={addButton}
        contained={policies.length > 0}
      >
        {isLoading ? (
          <div className={cn(tokens.surface.panel, tokens.surface.divider)}>
            {Array.from({ length: 2 }).map((_, i) => (
              <div key={i} className='px-3 py-3'>
                <div className='h-3 w-40 animate-pulse rounded bg-neutral-100 dark:bg-fk-raised' />
              </div>
            ))}
          </div>
        ) : isError ? (
          <EmptyState icon={ArrowLeftRight} compact label={t('token_exchange.error')} />
        ) : policies.length > 0 ? (
          <ul className={tokens.surface.divider}>
            {policies.map((policy) => (
              <li key={policy.id} className='flex items-center gap-3 py-2.5'>
                <div className='min-w-0 flex-1'>
                  <p className='font-mono-ui text-xs text-neutral-900 dark:text-neutral-100'>
                    {policy.target_audience}
                  </p>
                  <div className='mt-1 flex flex-wrap items-center gap-1.5'>
                    {policy.allowed_scopes && policy.allowed_scopes.length > 0 ? (
                      policy.allowed_scopes.map((scope) => (
                        <Pill key={scope} mono>
                          {scope}
                        </Pill>
                      ))
                    ) : (
                      <span className='text-xs text-neutral-400 dark:text-neutral-500'>
                        {t('token_exchange.list.no_scope_cap')}
                      </span>
                    )}
                  </div>
                </div>

                <Pill tone={policy.allow_impersonation ? 'success' : 'neutral'}>
                  {t('token_exchange.list.impersonation')}
                </Pill>
                <Pill tone={policy.allow_delegation ? 'success' : 'neutral'}>
                  {t('token_exchange.list.delegation')}
                </Pill>

                <Button
                  variant='ghost'
                  size='icon'
                  aria-label={t('token_exchange.list.delete', { name: policy.target_audience })}
                  onClick={() => onRequestDelete(policy)}
                  className='size-7 text-neutral-400 dark:text-neutral-500 hover:text-fk-danger'
                >
                  <Trash2 />
                </Button>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState
            icon={ArrowLeftRight}
            compact
            label={t('token_exchange.empty.label')}
            hint={t('token_exchange.empty.hint')}
            action={addButton}
          />
        )}
      </Section>

      <CreatePolicyDialog
        open={createOpen}
        onOpenChange={onCreateOpenChange}
        audiences={audiences}
        isCreating={isCreating}
        onCreate={onCreate}
      />

      <ConfirmDestructiveDialog
        open={pendingDelete !== null}
        onOpenChange={(open) => {
          if (!open) onCancelDelete()
        }}
        resourceName={pendingDelete?.target_audience ?? ''}
        title={t('token_exchange.delete.title')}
        description={t('token_exchange.delete.description', {
          name: pendingDelete?.target_audience ?? '',
        })}
        pending={isDeleting}
        onConfirm={onConfirmDelete}
      />
    </>
  )
}
