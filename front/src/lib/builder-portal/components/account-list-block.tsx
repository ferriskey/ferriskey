import type { CSSProperties } from 'react'
import { Plus, X } from 'lucide-react'
import type { BuilderNode } from '../../builder-core'

export interface AccountListEntry {
  sessionId: string
  displayName: string
  maskedEmail: string | null
  requiresReauthentication: boolean
}

export interface AccountChooserOptions {
  accounts: AccountListEntry[]
  isLoading: boolean
  busySessionId: string | null
  errorMessage: string | null
  onContinue: (sessionId: string) => void
  onRemove: (sessionId: string) => void
  onUseAnother: () => void
}

const NOOP = () => undefined

const PLACEHOLDER_ACCOUNT_CHOOSER: AccountChooserOptions = {
  accounts: [
    {
      sessionId: 'placeholder-1',
      displayName: 'Alice Martin',
      maskedEmail: 'a***@acme.com',
      requiresReauthentication: false,
    },
    {
      sessionId: 'placeholder-2',
      displayName: 'Bob Durand',
      maskedEmail: 'b***@acme.com',
      requiresReauthentication: true,
    },
  ],
  isLoading: false,
  busySessionId: null,
  errorMessage: null,
  onContinue: NOOP,
  onRemove: NOOP,
  onUseAnother: NOOP,
}

function initialOf(name: string): string {
  const first = Array.from(name.trim())[0]
  return first ? first.toUpperCase() : '?'
}

function rowStyle(enabled: boolean): CSSProperties {
  return {
    display: 'flex',
    alignItems: 'center',
    gap: 10,
    width: '100%',
    padding: '8px 10px',
    borderRadius: 'var(--fk-radius-input, 6px)',
    border: '1px solid rgba(0,0,0,0.08)',
    background: 'rgba(0,0,0,0.02)',
    color: 'var(--fk-color-body-text, #111827)',
    font: 'inherit',
    textAlign: 'left',
    cursor: enabled ? 'pointer' : 'default',
    opacity: enabled ? 1 : 0.6,
  }
}

const AVATAR_STYLE: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: 28,
  height: 28,
  flexShrink: 0,
  borderRadius: '50%',
  background: 'var(--fk-color-primary-button, #635dff)',
  color: 'var(--fk-color-primary-button-label, #ffffff)',
  fontSize: 12,
  fontWeight: 600,
}

const BADGE_STYLE: CSSProperties = {
  flexShrink: 0,
  padding: '2px 8px',
  borderRadius: 999,
  border: '1px solid rgba(0,0,0,0.12)',
  fontSize: 11,
  fontWeight: 500,
  color: 'var(--fk-color-body-text, #6b7280)',
}

const REMOVE_STYLE: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: 28,
  height: 28,
  flexShrink: 0,
  padding: 0,
  borderRadius: 'var(--fk-radius-input, 6px)',
  border: 'none',
  background: 'transparent',
  color: 'var(--fk-color-body-text, #6b7280)',
}

export function AccountListBlock({
  node,
  accountChooser,
  runtime,
  orderStyle,
}: {
  node: BuilderNode
  accountChooser?: AccountChooserOptions
  runtime?: boolean
  orderStyle: CSSProperties
}) {
  const chooser = accountChooser ?? (runtime ? null : PLACEHOLDER_ACCOUNT_CHOOSER)
  if (!chooser || chooser.isLoading) return null

  const busy = chooser.busySessionId !== null
  const enabled = Boolean(runtime) && !busy
  const useAnotherLabel = (node.props.useAnotherLabel as string) || 'Use another account'
  const signedOutLabel = (node.props.signedOutLabel as string) || 'Signed out'

  return (
    <div
      data-fk-id={node.id}
      style={{ display: 'flex', flexDirection: 'column', gap: 8, ...orderStyle }}
    >
      {chooser.errorMessage && (
        <div
          role='alert'
          style={{
            padding: '8px 10px',
            borderRadius: 'var(--fk-radius-input, 6px)',
            border: '1px solid var(--fk-color-error, #d03c38)',
            color: 'var(--fk-color-error, #d03c38)',
            fontSize: 13,
          }}
        >
          {chooser.errorMessage}
        </div>
      )}
      <ul
        style={{
          listStyle: 'none',
          margin: 0,
          padding: 0,
          display: 'flex',
          flexDirection: 'column',
          gap: 6,
        }}
      >
        {chooser.accounts.map((account) => (
          <li key={account.sessionId} style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <button
              type='button'
              disabled={!enabled}
              aria-busy={chooser.busySessionId === account.sessionId}
              onClick={() => chooser.onContinue(account.sessionId)}
              style={{ ...rowStyle(enabled), flex: 1, minWidth: 0 }}
            >
              <span aria-hidden style={AVATAR_STYLE}>
                {initialOf(account.displayName)}
              </span>
              <span style={{ display: 'flex', flexDirection: 'column', flex: 1, minWidth: 0 }}>
                <span
                  style={{
                    fontSize: 'var(--fk-font-base-size, 14px)',
                    fontWeight: 500,
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: 'nowrap',
                  }}
                >
                  {account.displayName}
                </span>
                {account.maskedEmail && (
                  <span
                    style={{
                      fontSize: 12,
                      opacity: 0.7,
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {account.maskedEmail}
                  </span>
                )}
              </span>
              {account.requiresReauthentication && <span style={BADGE_STYLE}>{signedOutLabel}</span>}
            </button>
            <button
              type='button'
              disabled={!enabled}
              aria-label={`Remove ${account.displayName}`}
              onClick={() => chooser.onRemove(account.sessionId)}
              style={{ ...REMOVE_STYLE, cursor: enabled ? 'pointer' : 'default' }}
            >
              <X size={14} aria-hidden />
            </button>
          </li>
        ))}
      </ul>
      <button
        type='button'
        disabled={!enabled}
        onClick={chooser.onUseAnother}
        style={{ ...rowStyle(enabled), borderStyle: 'dashed' }}
      >
        <span aria-hidden style={{ ...AVATAR_STYLE, background: 'transparent', color: 'inherit' }}>
          <Plus size={14} />
        </span>
        <span style={{ fontSize: 'var(--fk-font-base-size, 14px)', fontWeight: 500 }}>
          {useAnotherLabel}
        </span>
      </button>
    </div>
  )
}
