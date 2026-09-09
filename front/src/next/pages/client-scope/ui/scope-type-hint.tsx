export default function ScopeTypeHint({ scopeType }: { scopeType: 'optional' | 'default' }) {
  return (
    <p className='mt-1.5 text-xs text-neutral-500 dark:text-neutral-400'>
      {scopeType === 'default'
        ? 'Attached to every new client and issued without the client asking for it.'
        : 'Offered to clients, but issued only when they request it by name.'}
    </p>
  )
}
