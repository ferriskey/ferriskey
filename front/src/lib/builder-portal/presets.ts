import type { BuilderNode } from '@/lib/builder-core'
import { generateNodeId } from '@/lib/builder-core'
import { getDefaultNode } from './components'

/**
 * Ready-made block trees the admin can drop into a page in one click —
 * massively reduces the time-to-first-good-result for someone who's never
 * touched the builder. Each preset is a factory because nodes need freshly
 * generated ids on every insertion (you can stamp the same preset multiple
 * times in the same tree without collisions).
 */
export interface PortalPreset {
  id: string
  label: string
  description: string
  /** Generates a fresh subtree with unique ids on each call. */
  factory: () => BuilderNode[]
}

/**
 * Build a node from the type's default props/styles, override selected
 * fields, and stamp a fresh id. Optional `children` replaces (not merges
 * with) the default children — most blocks have none, but `card` ships
 * three slots in its default, so a preset that wants its own card layout
 * must pass `children` explicitly.
 */
function node(
  type: string,
  overrides: Partial<Omit<BuilderNode, 'id' | 'type'>> = {},
): BuilderNode {
  const base = getDefaultNode(type)
  return {
    ...base,
    ...overrides,
    type,
    id: generateNodeId(),
    children: overrides.children ?? base.children,
  }
}

function signInCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Welcome back' }),
            node('text', { content: 'Sign in to your account.' }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            // Surfaces invalid-credentials / session-expired messages
            // inline above the form — hidden when there's no error.
            node('form_error_banner'),
            node('email_input'),
            node('password_input'),
            node('submit_button', { content: 'Continue' }),
            node('forgot_password_link'),
            node('register_link'),
          ],
        }),
      ],
    }),
  ]
}

function registerCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '480px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Create your account' }),
            node('text', { content: 'It only takes a minute.' }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('first_name_input'),
            node('last_name_input'),
            node('email_input'),
            node('password_input'),
            node('password_confirm_input'),
            node('submit_button', { content: 'Create account' }),
            node('back_to_login_link', { content: 'Already have an account? Sign in' }),
          ],
        }),
      ],
    }),
  ]
}

function passwordlessCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Sign in without a password' }),
            node('text', {
              content: 'We will email you a magic link. No password to remember.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('email_input'),
            node('submit_button', { content: 'Send magic link' }),
            node('passkey_button', { content: 'Use a passkey instead' }),
          ],
        }),
      ],
    }),
  ]
}

function totpCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Two-factor authentication' }),
            node('text', {
              content: 'Enter the 6-digit code from your authenticator app.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('totp_input'),
            node('submit_button', { content: 'Verify' }),
          ],
        }),
      ],
    }),
  ]
}

function forgotPasswordCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Forgot your password?' }),
            node('text', {
              content:
                'Enter the email associated with your account and we will send you a reset link.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('email_input'),
            node('submit_button', { content: 'Send reset link' }),
            node('back_to_login_link'),
          ],
        }),
      ],
    }),
  ]
}

function resetPasswordCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Reset your password' }),
            node('text', {
              content:
                'Choose a new password for your account. You will use it to sign in next time.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('password_input', { props: { label: 'New password', placeholder: '', type: 'password', name: 'password' } }),
            node('password_confirm_input', { props: { label: 'Confirm new password', placeholder: '', type: 'password', name: 'password_confirm' } }),
            node('submit_button', { content: 'Update password' }),
            node('back_to_login_link'),
          ],
        }),
      ],
    }),
  ]
}

function magicLinkRequestCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Sign in by email' }),
            node('text', {
              content:
                'Enter your email address and we will send you a link to sign in.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('email_input'),
            node('submit_button', { content: 'Send magic link' }),
            node('back_to_login_link'),
          ],
        }),
      ],
    }),
  ]
}

function magicLinkVerifyCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Almost there' }),
            node('text', {
              content:
                'Click the button below to finish signing you in. The verification token from the email link is read automatically.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('submit_button', { content: 'Complete sign-in' }),
            node('back_to_login_link'),
          ],
        }),
      ],
    }),
  ]
}

function totpSetupCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '480px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Set up two-factor authentication' }),
            node('text', {
              content:
                'Scan the QR code with an authenticator app (Authy, Google Authenticator, 1Password, …) then enter the 6-digit code it generates.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '16px' },
          children: [
            node('totp_qr_code'),
            node('totp_secret'),
            node('totp_input', {
              props: { label: 'Code from your app', name: 'totp' },
            }),
            node('input', {
              props: {
                label: 'Device name (optional)',
                placeholder: 'e.g. iPhone',
                type: 'text',
                name: 'device_name',
              },
            }),
            node('submit_button', { content: 'Confirm' }),
          ],
        }),
      ],
    }),
  ]
}

function emailVerifiedCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Email verified' }),
            node('text', {
              content:
                'Your email address has been confirmed. You can now sign in to your account.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            node('back_to_login_link', { content: 'Continue to sign in' }),
          ],
        }),
      ],
    }),
  ]
}

function verifyEmailCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Verify your email' }),
            node('text', {
              content:
                'We sent a verification link to your email address. Check your inbox (and spam folder) and click the link to finish signing in. Didn\u2019t get it?',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [
            // The submit button now resends the verification email. The
            // portal-page submit hook wires `verify_email` → resend
            // mutation (token read from URL `client_data`), so a click
            // here calls the API and toasts the result.
            node('submit_button', { content: 'Resend verification email' }),
            node('back_to_login_link'),
          ],
        }),
      ],
    }),
  ]
}

function socialOnlyCard(): BuilderNode[] {
  return [
    node('card', {
      props: { maxWidth: '440px', align: 'center' },
      children: [
        node('card-header', {
          props: { textAlign: 'center', gap: '6px' },
          children: [
            node('heading', { content: 'Continue with your provider' }),
            node('text', {
              content: 'Pick one of the providers below to sign in.',
            }),
          ],
        }),
        node('card-content', {
          props: { gap: '12px' },
          children: [node('identity_providers')],
        }),
      ],
    }),
  ]
}

function centeredHeader(): BuilderNode[] {
  return [
    node('container', {
      props: {
        direction: 'column',
        align: 'center',
        gap: '8px',
        padding: '24px',
      },
      children: [
        node('heading', { content: 'Welcome' }),
        node('text', { content: 'A short description goes here.' }),
      ],
    }),
  ]
}

function orContinueWith(): BuilderNode[] {
  return [
    node('container', {
      props: {
        direction: 'column',
        align: 'stretch',
        gap: '12px',
        padding: '0px',
      },
      children: [node('divider'), node('identity_providers')],
    }),
  ]
}

export const PORTAL_PRESETS: PortalPreset[] = [
  {
    id: 'sign-in-card',
    label: 'Sign-in card',
    description: 'Card with email + password + submit. Best for the Login page.',
    factory: signInCard,
  },
  {
    id: 'register-card',
    label: 'Register card',
    description:
      'Card with first/last name + email + password + confirm + submit. Best for the Register page.',
    factory: registerCard,
  },
  {
    id: 'totp-card',
    label: 'TOTP challenge card',
    description: 'Card with a 6-digit code input + verify button. For the TOTP page.',
    factory: totpCard,
  },
  {
    id: 'forgot-password-card',
    label: 'Forgot password card',
    description: 'Card with email + send-reset-link button. For the Forgot password page.',
    factory: forgotPasswordCard,
  },
  {
    id: 'reset-password-card',
    label: 'Reset password card',
    description:
      'Card with new password + confirm + update button. For the Reset password page (reached from the email link).',
    factory: resetPasswordCard,
  },
  {
    id: 'magic-link-request-card',
    label: 'Magic link request card',
    description: 'Card with email + send-magic-link button. For the Magic link request page.',
    factory: magicLinkRequestCard,
  },
  {
    id: 'magic-link-verify-card',
    label: 'Magic link verify card',
    description:
      'Card with a complete-sign-in button — the token comes from the email link URL. For the Magic link verify page.',
    factory: magicLinkVerifyCard,
  },
  {
    id: 'verify-email-card',
    label: 'Verify email card',
    description:
      'Card prompting the user to follow the verification link from their inbox. For the Verify email page.',
    factory: verifyEmailCard,
  },
  {
    id: 'email-verified-card',
    label: 'Email verified card',
    description:
      'Success card shown after a verification link has been clicked. For the Email verified page.',
    factory: emailVerifiedCard,
  },
  {
    id: 'totp-setup-card',
    label: 'TOTP setup card',
    description:
      'Card with QR code + secret fallback + code confirmation + device label. For the TOTP setup page.',
    factory: totpSetupCard,
  },
  {
    id: 'passwordless-card',
    label: 'Passwordless card',
    description: 'Card with email + magic link + passkey buttons.',
    factory: passwordlessCard,
  },
  {
    id: 'social-only-card',
    label: 'Social-only card',
    description: 'Card with only the identity providers list, no password.',
    factory: socialOnlyCard,
  },
  {
    id: 'centered-header',
    label: 'Centered header',
    description: 'Heading + subtitle, centered. Drop above an existing form.',
    factory: centeredHeader,
  },
  {
    id: 'or-continue-with',
    label: '"Or continue with" block',
    description: 'Divider + identity providers, ready to append at the bottom.',
    factory: orContinueWith,
  },
]
