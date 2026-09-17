import { useState } from 'react'
import { toast } from 'sonner'
import { Check, Copy, KeyRound } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/components/kit'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useRotateWebhookSecret } from '@/api/webhook.api'
import { apiErrorMessage } from '@/lib/api-error'

export interface WebhookSigningSecretProps {
  realm: string
  webhookId: string
}

export default function WebhookSigningSecret({ realm, webhookId }: WebhookSigningSecretProps) {
  const { t } = useTranslation('webhook')
  const [open, setOpen] = useState(false)
  const [secret, setSecret] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)

  const { mutate: rotateSecret, isPending } = useRotateWebhookSecret()

  const close = () => {
    setOpen(false)
    setSecret(null)
    setCopied(false)
  }

  const generate = () => {
    rotateSecret(
      { path: { realm_name: realm, webhook_id: webhookId } },
      {
        onSuccess: (data) => setSecret(data.secret),
        onError: (error: unknown) => {
          toast.error(apiErrorMessage(error, 'Could not generate a new secret'))
          close()
        },
      }
    )
  }

  const copy = async () => {
    if (!secret) return

    try {
      await navigator.clipboard.writeText(secret)
      setCopied(true)
    } catch {
      toast.error(t('secret.toast.copy_failed'))
    }
  }

  return (
    <>
      <div className='flex h-full items-center'>
        <Button variant='outline' onClick={() => setOpen(true)}>
          <KeyRound className='mr-1.5 size-3.5' />
          {t('secret.generate')}
        </Button>
      </div>

      <Dialog open={open} onOpenChange={(next) => (next ? setOpen(true) : close())}>
        <DialogContent className='max-w-lg'>
          {secret === null ? (
            <>
              <DialogHeader>
                <DialogTitle>{t('secret.confirm.title')}</DialogTitle>
                <DialogDescription>{t('secret.confirm.description')}</DialogDescription>
              </DialogHeader>
              <DialogFooter>
                <Button variant='outline' onClick={close} disabled={isPending}>
                  {t('secret.confirm.cancel')}
                </Button>
                <Button onClick={generate} disabled={isPending}>
                  {isPending ? t('secret.confirm.pending') : t('secret.confirm.action')}
                </Button>
              </DialogFooter>
            </>
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>{t('secret.reveal.title')}</DialogTitle>
                <DialogDescription>{t('secret.reveal.description')}</DialogDescription>
              </DialogHeader>

              <div className='flex items-center gap-2'>
                <code className='min-w-0 flex-1 overflow-x-auto rounded-md border border-fk-line bg-neutral-50 px-3 py-2 font-mono-ui text-[12px] dark:bg-fk-raised'>
                  {secret}
                </code>
                <Button variant='outline' size='sm' onClick={copy}>
                  {copied ? (
                    <>
                      <Check className='mr-1 size-3.5' />
                      {t('secret.reveal.copied')}
                    </>
                  ) : (
                    <>
                      <Copy className='mr-1 size-3.5' />
                      {t('secret.reveal.copy')}
                    </>
                  )}
                </Button>
              </div>

              <DialogFooter>
                <Button onClick={close}>{t('secret.reveal.done')}</Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
}
