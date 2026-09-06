import { Outlet, useLocation, useNavigate, useParams } from 'react-router'
import { RouterParams } from '@/routes/router'

export default function AccountLayout() {
  const navigate = useNavigate()
  const { realm_name } = useParams<RouterParams>()
  const { pathname } = useLocation()

  const baseUrl = `/realms/${realm_name}/account`

  const tabs = [
    { key: 'overview', label: 'Personal info', path: baseUrl },
    { key: 'sessions', label: 'Sessions', path: `${baseUrl}/sessions` },
  ]

  return (
    <div className='flex flex-col gap-6 p-8'>
      <div className='-mx-8 -mt-8 px-8 pt-8 pb-4 border-b'>
        <h1 className='text-2xl font-bold tracking-tight'>My Account</h1>
        <p className='text-sm text-muted-foreground mt-1'>
          Manage your own profile and active sessions.
        </p>
      </div>

      <div className='-mx-8 px-8 pb-4 border-b flex items-center gap-2 -mt-2'>
        {tabs.map((tab) => {
          const isActive =
            tab.key === 'overview' ? pathname === baseUrl : pathname.startsWith(tab.path)
          return (
            <button
              key={tab.key}
              onClick={() => navigate(tab.path)}
              className={`px-4 py-1.5 rounded-md text-sm font-medium transition-colors border ${
                isActive
                  ? 'bg-primary/10 text-primary border-primary/40'
                  : 'bg-transparent text-foreground border-border hover:bg-muted'
              }`}
            >
              {tab.label}
            </button>
          )
        })}
      </div>

      <Outlet />
    </div>
  )
}
