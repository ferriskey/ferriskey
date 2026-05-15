import { Monitor, Smartphone, Tablet } from 'lucide-react'
import { useMemo, useState, type CSSProperties } from 'react'
import {
  BuilderProvider,
  BuilderShell,
  Canvas,
  ComponentLibrary,
  ConfigPanel,
  type BuilderNode,
} from '@/lib/builder-core'
import { createPortalAdapter, getDefaultNode } from '@/lib/builder-portal'
import { defaultTheme, themeToCssVars } from '@/pages/portal-theme/lib/theme'

type Viewport = 'desktop' | 'tablet' | 'mobile'

const VIEWPORT_WIDTHS: Record<Viewport, number> = {
  desktop: 1024,
  tablet: 768,
  mobile: 375,
}

const seedTree = (): BuilderNode[] => {
  const shell = getDefaultNode('container')
  const card = getDefaultNode('container')
  const header = getDefaultNode('container')
  const altButtons = getDefaultNode('container')
  const logo = getDefaultNode('image')
  const caption = getDefaultNode('text')
  const title = getDefaultNode('heading')
  const usernameInput = getDefaultNode('input')
  const passwordInput = getDefaultNode('input')
  const loginButton = getDefaultNode('button')
  const divider = getDefaultNode('divider')
  const passkeyButton = getDefaultNode('button')
  const magicLinkButton = getDefaultNode('button')
  const signupText = getDefaultNode('text')

  return [
    {
      id: 'login-shell',
      ...shell,
      props: {
        ...shell.props,
        direction: 'column',
        align: 'center',
        gap: '0px',
        padding: '40px 16px',
      },
      children: [
        {
          id: 'login-card',
          ...card,
          props: {
            ...card.props,
            direction: 'column',
            gap: '24px',
            padding: '40px',
            backgroundColor: '#ffffff',
            borderRadius: 'var(--fk-radius-widget, 12px)',
            width: '440px',
          },
          children: [
            {
              id: 'login-header',
              ...header,
              props: {
                ...header.props,
                direction: 'row',
                align: 'center',
                gap: '12px',
                padding: '0px',
              },
              children: [
                {
                  id: 'login-logo',
                  ...logo,
                  props: { ...logo.props, src: '/logo_ferriskey.png', alt: 'FerrisKey', width: '28px', height: '28px', align: 'left' },
                },
                {
                  id: 'login-caption',
                  ...caption,
                  content: 'FERRISKEY',
                  props: { ...caption.props, fontSize: '12px', fontWeight: '600', textAlign: 'left', color: '#6b7280' },
                },
              ],
            },
            {
              id: 'login-title',
              ...title,
              content: 'MASTER',
              props: { ...title.props, level: '1', textAlign: 'left', fontSize: '30px', fontWeight: '600' },
            },
            {
              id: 'login-username',
              ...usernameInput,
              props: { label: 'Username', placeholder: '', type: 'text' },
            },
            {
              id: 'login-password',
              ...passwordInput,
              props: { label: 'Password', placeholder: '', type: 'password', helperText: 'Forgot your password?' },
            },
            {
              id: 'login-submit',
              ...loginButton,
              content: 'Login',
              props: { ...loginButton.props, variant: 'primary', fullWidth: 'true' },
            },
            {
              id: 'login-divider',
              ...divider,
              props: { ...divider.props, color: '#e5e7eb', thickness: '1px', width: '100%' },
            },
            {
              id: 'login-alt-buttons',
              ...altButtons,
              props: {
                ...altButtons.props,
                direction: 'column',
                gap: '8px',
                padding: '0px',
              },
              children: [
                {
                  id: 'login-passkey',
                  ...passkeyButton,
                  content: 'Sign in with a passkey',
                  props: { ...passkeyButton.props, variant: 'outline', fullWidth: 'true' },
                },
                {
                  id: 'login-magic-link',
                  ...magicLinkButton,
                  content: 'Sign in with a magic link',
                  props: { ...magicLinkButton.props, variant: 'outline', fullWidth: 'true' },
                },
              ],
            },
            {
              id: 'login-signup',
              ...signupText,
              content: 'Don’t have an account? Sign up',
              props: { ...signupText.props, fontSize: '13px', textAlign: 'center', color: '#6b7280' },
            },
          ],
        },
      ],
    },
  ]
}

export default function PagePortalBuilderDemo() {
  const adapter = useMemo(() => createPortalAdapter(), [])
  const [tree, setTree] = useState<BuilderNode[]>(() => seedTree())
  const [viewport, setViewport] = useState<Viewport>('desktop')
  const cssVars = useMemo(() => themeToCssVars(defaultTheme) as CSSProperties, [])

  return (
    <BuilderProvider adapter={adapter} initialTree={tree} onChange={setTree}>
      <div className='flex h-[calc(100vh-3rem)] w-full min-w-0 flex-col overflow-hidden'>
        <header className='flex items-center justify-between border-b border-border px-6 py-3'>
          <div>
            <h1 className='text-lg font-semibold'>Builder demo</h1>
            <p className='text-xs text-muted-foreground'>
              Sandbox for the builder-portal adapter — not persisted.
            </p>
          </div>
          <div className='flex items-center gap-1 rounded-md border border-border p-0.5'>
            <ViewportButton active={viewport === 'desktop'} onClick={() => setViewport('desktop')} label='Desktop'>
              <Monitor size={14} />
            </ViewportButton>
            <ViewportButton active={viewport === 'tablet'} onClick={() => setViewport('tablet')} label='Tablet'>
              <Tablet size={14} />
            </ViewportButton>
            <ViewportButton active={viewport === 'mobile'} onClick={() => setViewport('mobile')} label='Mobile'>
              <Smartphone size={14} />
            </ViewportButton>
          </div>
        </header>

        <BuilderShell>
          <div className='flex min-w-0 flex-1 overflow-hidden'>
            <div className='w-56 shrink-0 overflow-y-auto border-r border-border'>
              <ComponentLibrary />
            </div>

            <div
              className='flex min-w-0 flex-1 justify-center overflow-auto bg-muted/30 p-6'
              style={cssVars}
            >
              <div
                className='shrink-0 self-start rounded-lg border border-border bg-background shadow-sm transition-all duration-200'
                style={{ width: VIEWPORT_WIDTHS[viewport] }}
              >
                <Canvas maxWidth={VIEWPORT_WIDTHS[viewport]} />
              </div>
            </div>

            <div className='w-80 shrink-0 overflow-y-auto border-l border-border'>
              <ConfigPanel />
            </div>
          </div>
        </BuilderShell>
      </div>
    </BuilderProvider>
  )
}

function ViewportButton({
  active,
  onClick,
  label,
  children,
}: {
  active: boolean
  onClick: () => void
  label: string
  children: React.ReactNode
}) {
  return (
    <button
      type='button'
      title={label}
      className={`rounded px-2 py-1 text-xs transition-colors ${
        active ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted'
      }`}
      onClick={onClick}
    >
      {children}
    </button>
  )
}
