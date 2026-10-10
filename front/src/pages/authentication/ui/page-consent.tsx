import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'
import { BasicSpinner } from '@/components/ui/spinner'
import { AlertCircle, Lock } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import type { ConsentScopeView } from '../utils/consent-decision'
import './page-login.css'
import { AUTH_NAMESPACE, BRAND_NAME } from '../constants'

export interface PageConsentProps {
  clientName: string
  clientUriHost?: string | null
  defaultScopes: ConsentScopeView[]
  optionalScopes: ConsentScopeView[]
  approvedOptional: ReadonlySet<string>
  onToggleScope: (name: string) => void
  onAllow: () => void
  onDeny: () => void
  isLoading: boolean
  isAllowing: boolean
  isDenying: boolean
  errorMessage: string | null
}

export default function PageConsent({
  clientName,
  clientUriHost,
  defaultScopes,
  optionalScopes,
  approvedOptional,
  onToggleScope,
  onAllow,
  onDeny,
  isLoading,
  isAllowing,
  isDenying,
  errorMessage,
}: PageConsentProps) {
  const { t } = useTranslation(AUTH_NAMESPACE)
  const isSubmitting = isAllowing || isDenying

  return (
    <div className='login-shell relative flex min-h-svh items-center justify-center px-6 py-10'>
      <div className='relative z-10 w-full max-w-sm md:max-w-md lg:max-w-lg'>
        <div className='flex flex-col gap-6'>
          <Card className='login-card overflow-hidden border p-0 shadow-sm'>
            <CardContent className='grid gap-0 p-0'>
              <div className='p-8 md:p-10'>
                <div className='flex flex-col gap-7'>
                  <div className='space-y-2'>
                    <div className='flex items-center gap-3'>
                      <img
                        src='/logo_ferriskey.png'
                        alt={BRAND_NAME}
                        className='h-7 w-7 object-contain'
                      />
                      <p className='text-xs font-semibold uppercase tracking-[0.35em] text-muted-foreground'>
                        {BRAND_NAME}
                      </p>
                    </div>
                    <h1 className='login-title text-3xl font-semibold tracking-tight text-foreground'>
                      {t('consent.title')}
                    </h1>
                    {clientName && (
                      <p className='text-sm text-muted-foreground'>
                        <Trans
                          i18nKey={`${AUTH_NAMESPACE}:consent.request`}
                          values={{ name: clientName }}
                          components={{ app: <span className='font-medium text-foreground' /> }}
                        />
                      </p>
                    )}
                    {clientName && clientUriHost && (
                      <p className='font-mono-ui text-xs text-muted-foreground'>
                        {t('consent.client_host', { host: clientUriHost })}
                      </p>
                    )}
                  </div>

                  {errorMessage && (
                    <div
                      role='alert'
                      className='flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 p-3 text-sm text-destructive'
                    >
                      <AlertCircle className='mt-0.5 h-4 w-4 shrink-0' />
                      <span>{errorMessage}</span>
                    </div>
                  )}

                  {isLoading ? (
                    <div className='flex justify-center py-4'>
                      <BasicSpinner />
                    </div>
                  ) : (
                    <>
                      <div className='flex flex-col gap-2'>
                        {defaultScopes.length > 0 && (
                          <p className='text-xs uppercase tracking-wide text-muted-foreground'>
                            {t('consent.default_scopes_label')}
                          </p>
                        )}
                        {defaultScopes.map((scope) => (
                          <div
                            key={scope.name}
                            className='flex items-center gap-3 rounded-lg border border-border p-3'
                          >
                            <Lock className='h-4 w-4 shrink-0 text-muted-foreground' />
                            <div className='flex flex-col'>
                              <span className='text-sm font-medium text-foreground'>
                                {scope.name}
                              </span>
                              {scope.description && (
                                <span className='text-xs text-muted-foreground'>
                                  {scope.description}
                                </span>
                              )}
                            </div>
                          </div>
                        ))}
                        {optionalScopes.length > 0 && (
                          <p className='pt-2 text-xs uppercase tracking-wide text-muted-foreground'>
                            {t('consent.optional_scopes_label')}
                          </p>
                        )}
                        {optionalScopes.map((scope) => (
                          <div
                            key={scope.name}
                            className='flex items-center justify-between gap-3 rounded-lg border border-border p-3'
                          >
                            <div className='flex flex-col'>
                              <span className='text-sm font-medium text-foreground'>
                                {scope.name}
                              </span>
                              {scope.description && (
                                <span className='text-xs text-muted-foreground'>
                                  {scope.description}
                                </span>
                              )}
                            </div>
                            <Switch
                              checked={approvedOptional.has(scope.name)}
                              onCheckedChange={() => onToggleScope(scope.name)}
                              disabled={isSubmitting}
                            />
                          </div>
                        ))}
                      </div>

                      <div className='flex flex-col gap-2'>
                        <Button
                          type='button'
                          className='w-full rounded-lg py-5 text-sm'
                          disabled={isSubmitting}
                          onClick={onAllow}
                        >
                          {isAllowing ? (
                            <div className='flex items-center gap-2'>
                              <BasicSpinner />
                              <span>{t('consent.allowing')}</span>
                            </div>
                          ) : (
                            t('consent.allow')
                          )}
                        </Button>
                        <Button
                          type='button'
                          variant='outline'
                          className='w-full rounded-lg py-5 text-sm'
                          disabled={isSubmitting}
                          onClick={onDeny}
                        >
                          {isDenying ? (
                            <div className='flex items-center gap-2'>
                              <BasicSpinner />
                              <span>{t('consent.denying')}</span>
                            </div>
                          ) : (
                            t('consent.deny')
                          )}
                        </Button>
                      </div>
                    </>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
