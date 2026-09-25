import { zodResolver } from '@hookform/resolvers/zod'
import { useCallback, useState } from 'react'
import { useForm } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { useSendMagicLink } from '@/api/trident.api'
import { magicLinkSchema, MagicLinkSchema } from '@/pages/authentication/schemas/magic-link.schema'
import type { MagicLinkStep } from '../ui/page-login'
import { AUTH_NAMESPACE } from '../constants'

type Options = {
  realm_name: string | undefined
  getRememberMe: () => boolean
}

export function useMagicLinkAuth({ realm_name, getRememberMe }: Options) {
  const { t } = useTranslation(AUTH_NAMESPACE)
  const { mutate: sendMagicLink, isPending: isMagicLinkLoading } = useSendMagicLink()
  const [magicLinkStep, setMagicLinkStep] = useState<MagicLinkStep>('idle')

  const magicLinkForm = useForm<MagicLinkSchema>({
    resolver: zodResolver(magicLinkSchema),
    defaultValues: { email: '' },
  })

  const onMagicLinkLogin = useCallback(() => {
    setMagicLinkStep('form')
  }, [])

  const onMagicLinkBack = useCallback(() => {
    setMagicLinkStep('idle')
    magicLinkForm.reset()
  }, [magicLinkForm])

  const onMagicLinkSubmit = useCallback(
    (data: MagicLinkSchema) => {
      sendMagicLink(
        {
          path: { realm_name: realm_name ?? 'master' },
          body: { email: data.email, remember_me: getRememberMe() },
        },
        {
          onSuccess: () => setMagicLinkStep('sent'),
          onError: () => toast.error(t('magic_link.toast.send_failed')),
        }
      )
    },
    [realm_name, sendMagicLink, getRememberMe, t]
  )

  return {
    magicLinkForm,
    magicLinkStep,
    isMagicLinkLoading,
    onMagicLinkLogin,
    onMagicLinkBack,
    onMagicLinkSubmit,
  }
}
