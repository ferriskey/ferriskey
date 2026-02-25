import { Realm } from '@/api/core.interface'
import { cn } from '@/lib/utils'
import { REALM_SETTINGS_URL } from '@/routes/router'
import { Outlet, useNavigate } from 'react-router-dom'

interface PageRealmSettingsProps {
  realm: Realm
  tab?: string
  setTab?: (value: string) => void
}

export default function PageRealmSettings({ realm, tab, setTab }: PageRealmSettingsProps) {
  const navigate = useNavigate()

  const tabs = [
    { key: 'general', label: 'General' },
    { key: 'login', label: 'Login' },
    { key: 'webhooks', label: 'Webhooks' },
    { key: 'security', label: 'Security', disabled: true },
  ]

  const handleTabChange = (key: string) => {
    navigate(`${REALM_SETTINGS_URL(realm.name)}/${key}`)
    setTab?.(key)
  }

  return (
    <div className='flex flex-col gap-6 p-8'>
      {/* Header */}
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b flex items-start justify-between gap-4'>
        <div>
          <h1 className='text-2xl font-bold tracking-tight'>{realm.name}</h1>
          <p className='text-sm text-muted-foreground mt-1'>
            Manage realm settings — users, applications, roles, and groups.
          </p>
        </div>
        <span className='inline-flex items-center px-2.5 py-0.5 rounded-md border border-border text-muted-foreground text-xs font-mono bg-muted/50'>
          {realm.id}
        </span>
      </div>

      {/* Tabs */}
      <div className='-mx-8 px-8 pb-4 border-b flex items-center gap-2 -mt-2'>
        {tabs.map((t) => (
          <button
            key={t.key}
            onClick={() => !t.disabled && handleTabChange(t.key)}
            disabled={t.disabled}
            className={cn('px-4 py-1.5 rounded-md text-sm font-medium transition-colors border',
              t.disabled
                ? 'bg-transparent text-muted-foreground border-border opacity-50 cursor-not-allowed'
                : tab === t.key
                  ? 'bg-primary/10 text-primary border-primary/40'
                  : 'bg-transparent text-foreground border-border hover:bg-muted'
            )}
          >
            {t.label}
          </button>
        ))}
      </div>

      <Outlet />
    </div>
  )
}
