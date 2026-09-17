import { useState } from 'react'
import { toast } from 'sonner'
import { Check, Copy, KeyRound } from 'lucide-react'
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

export interface WebhookSigningSecretProps {
  realm: string
  webhookId: string
}

export default function WebhookSigningSecret({ realm, webhookId }: WebhookSigningSecretProps) {
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
          const body = error as { message?: string }
          toast.error(body?.message ?? 'Could not generate a new secret')
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
      toast.error('Could not copy. Select the secret and copy it manually.')
    }
  }

  return (
    <>
      <div className='flex h-full items-center'>
        <Button variant='outline' onClick={() => setOpen(true)}>
          <KeyRound className='mr-1.5 size-3.5' />
          Generate a new secret
        </Button>
      </div>

      <Dialog open={open} onOpenChange={(next) => (next ? setOpen(true) : close())}>
        <DialogContent className='max-w-lg'>
          {secret === null ? (
            <>
              <DialogHeader>
                <DialogTitle>Generate a new signing secret?</DialogTitle>
                <DialogDescription>
                  The current secret stops working immediately. Any receiver still verifying
                  signatures with it will reject your deliveries until you give it the new one.
                </DialogDescription>
              </DialogHeader>
              <DialogFooter>
                <Button variant='outline' onClick={close} disabled={isPending}>
                  Cancel
                </Button>
                <Button onClick={generate} disabled={isPending}>
                  {isPending ? 'Generating…' : 'Generate'}
                </Button>
              </DialogFooter>
            </>
          ) : (
            <>
              <DialogHeader>
                <DialogTitle>Your new signing secret</DialogTitle>
                <DialogDescription>
                  Copy it now — this is the only time it is shown. Closing this dialog is the last
                  chance; it cannot be read again afterwards.
                </DialogDescription>
              </DialogHeader>

              <div className='flex items-center gap-2'>
                <code className='min-w-0 flex-1 overflow-x-auto rounded-md border border-fk-line bg-neutral-50 px-3 py-2 font-mono-ui text-[12px] dark:bg-fk-raised'>
                  {secret}
                </code>
                <Button variant='outline' size='sm' onClick={copy}>
                  {copied ? (
                    <>
                      <Check className='mr-1 size-3.5' />
                      Copied
                    </>
                  ) : (
                    <>
                      <Copy className='mr-1 size-3.5' />
                      Copy
                    </>
                  )}
                </Button>
              </div>

              <DialogFooter>
                <Button onClick={close}>Done</Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  )
}
