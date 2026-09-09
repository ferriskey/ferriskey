import { useState } from 'react'
import { Check, Copy } from 'lucide-react'
import { Button } from '@/components/kit/button'
import { FieldRow, Section } from '@/components/kit'
import { cn } from '@/lib/utils'

export interface QuickstartEndpoint {
  key: string
  label: string
  description: string
  value: string
}

export interface ApplicationQuickstartTabProps {
  clientId: string
  endpoints: QuickstartEndpoint[]
  snippetTitle: string
  snippetDescription: string
  snippet: string
}

function CopyButton({ value, label }: { value: string; label: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <Button
      type='button'
      variant='outline'
      size='icon'
      aria-label={label}
      onClick={() => void handleCopy()}
    >
      {copied ? <Check className='size-4 text-fk-success' /> : <Copy className='size-4' />}
    </Button>
  )
}

function CopyValue({ value, label }: { value: string; label: string }) {
  return (
    <div className='flex max-w-2xl items-center gap-2'>
      <code
        className={cn(
          'flex h-9 min-w-0 flex-1 items-center overflow-x-auto rounded-md border border-fk-line bg-neutral-50 px-2.5 font-mono-ui text-xs text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'
        )}
      >
        {value}
      </code>
      <CopyButton value={value} label={label} />
    </div>
  )
}

export default function ApplicationQuickstartTab({
  clientId,
  endpoints,
  snippetTitle,
  snippetDescription,
  snippet,
}: ApplicationQuickstartTabProps) {
  return (
    <>
      <Section
        title='Identity'
        description='What your application sends on every request.'
      >
        <FieldRow
          label='Client ID'
          description='Public: it travels in the browser on every sign-in. It is not a credential.'
        >
          <CopyValue value={clientId} label='Copy the client ID' />
        </FieldRow>
      </Section>

      <Section
        title='Endpoints'
        description='Point your OIDC or OAuth library at these URLs. Most libraries only need the discovery document.'
      >
        {endpoints.map((endpoint) => (
          <FieldRow key={endpoint.key} label={endpoint.label} description={endpoint.description}>
            <CopyValue value={endpoint.value} label={`Copy the ${endpoint.label} URL`} />
          </FieldRow>
        ))}
      </Section>

      <Section title={snippetTitle} description={snippetDescription}>
        <div className='relative py-4'>
          <pre className='overflow-x-auto rounded-md border border-fk-line bg-neutral-50 p-4 font-mono-ui text-xs leading-relaxed text-neutral-700 dark:bg-fk-surface dark:text-neutral-300'>
            {snippet}
          </pre>
          <div className='absolute right-2 top-6'>
            <CopyButton value={snippet} label='Copy the snippet' />
          </div>
        </div>
      </Section>
    </>
  )
}
