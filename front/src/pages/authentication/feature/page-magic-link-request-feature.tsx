import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useParams, useSearchParams } from 'react-router'
import { toast } from 'sonner'
import { useSendMagicLink } from '@/api/trident.api'
import {
  magicLinkSchema,
  type MagicLinkSchema,
} from '@/pages/authentication/schemas/magic-link.schema'
import PageMagicLinkRequest from '../ui/page-magic-link-request'
import { Form } from '@/components/ui/form'

/**
 * Hosts the dedicated magic-link request page. The portal layout wrapper
 * navigates here (with the user's pre-filled email passed via `?email=` when
 * possible) so the inline magic-link button always lands on a focused form,
 * even when the realm's login screen is fully custom-built.
 */
export default function PageMagicLinkRequestFeature() {
  const { realm_name } = useParams()
  const [searchParams] = useSearchParams()
  const [sent, setSent] = useState(false)

  const { mutate: sendMagicLink, isPending } = useSendMagicLink()

  const form = useForm<MagicLinkSchema>({
    resolver: zodResolver(magicLinkSchema),
    defaultValues: { email: searchParams.get('email') ?? '' },
  })

  // Re-sync the default email if the user navigates to this page with a
  // different `?email=` value (e.g., re-clicking the magic-link button after
  // changing the field on the previous page).
  useEffect(() => {
    const prefilled = searchParams.get('email')
    if (prefilled && form.getValues('email') !== prefilled) {
      form.reset({ email: prefilled })
    }
    // We intentionally depend only on the URL value — `form` is a stable ref.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [searchParams])

  function onSubmit(data: MagicLinkSchema) {
    sendMagicLink(
      {
        path: { realm_name: realm_name ?? 'master' },
        body: { email: data.email },
      },
      {
        onSuccess: () => setSent(true),
        onError: () => toast.error('Failed to send magic link'),
      },
    )
  }

  return (
    <Form {...form}>
      <PageMagicLinkRequest
        form={form}
        onSubmit={onSubmit}
        sent={sent}
        isPending={isPending}
      />
    </Form>
  )
}
