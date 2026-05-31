import {
  AlertCircle,
  ArrowLeft,
  AtSign,
  Box,
  CheckSquare,
  CreditCard,
  Fingerprint,
  Globe,
  HelpCircle,
  Heading as HeadingIcon,
  Hash,
  IdCard,
  Image as ImageIcon,
  KeyRound,
  LayoutTemplate,
  LockKeyhole,
  Mail,
  Minus,
  MousePointerClick,
  MoveVertical,
  PanelBottom,
  PanelTop,
  QrCode,
  ShieldCheck,
  Square,
  TextCursorInput,
  Type,
  User,
  UserPlus,
} from 'lucide-react'
import { generateNodeId, type BuilderNode, type ComponentDefinition } from '../builder-core'

const ALL_CHILDREN = [
  'container',
  'div',
  'card',
  'heading',
  'text',
  'image',
  'spacer',
  'divider',
  'button',
  'input',
  'username_input',
  'first_name_input',
  'last_name_input',
  'email_input',
  'password_input',
  'password_confirm_input',
  'totp_input',
  'submit_button',
  'magic_link_button',
  'passkey_button',
  'identity_providers',
  'forgot_password_link',
  'back_to_login_link',
  'register_link',
  'totp_qr_code',
  'totp_secret',
  'form_error_banner',
  'page-content',
]

/**
 * Children allowed inside a Card slot (header / content / footer). Same as
 * `ALL_CHILDREN` minus the slot blocks themselves and `card` (nesting cards
 * inside a slot would defeat the centred layout the parent card provides).
 */
const CARD_SLOT_CHILDREN = ALL_CHILDREN.filter((t) => t !== 'card')

export const portalComponents: ComponentDefinition[] = [
  {
    type: 'container',
    label: 'Container',
    icon: <Box size={14} />,
    isContainer: true,
    allowedChildren: ALL_CHILDREN,
    defaultProps: {
      direction: 'column',
      align: 'stretch',
      gap: '12px',
      padding: '16px',
    },
    defaultStyles: {},
  },
  {
    type: 'div',
    label: 'Div',
    icon: <Square size={14} />,
    isContainer: true,
    allowedChildren: ALL_CHILDREN,
    defaultProps: {
      // Display switches between block / flex / grid layouts.
      display: 'block',
      // Position / positioning offsets.
      position: 'static',
      top: '',
      right: '',
      bottom: '',
      left: '',
      zIndex: '',
      // Size.
      width: '',
      height: '',
      minWidth: '',
      maxWidth: '',
      minHeight: '',
      maxHeight: '',
      // Spacing — no default padding so a fresh Div is a truly empty box.
      padding: '',
      margin: '',
      gap: '',
      // Visual.
      backgroundColor: '',
      borderRadius: '',
      overflow: 'visible',
      // Flex props (used when display === 'flex').
      direction: 'row',
      wrap: 'nowrap',
      justifyContent: 'flex-start',
      alignItems: 'stretch',
      alignContent: 'stretch',
      // Grid props (used when display === 'grid').
      templateColumns: 'repeat(2, 1fr)',
      templateRows: '',
      columnGap: '',
      rowGap: '',
      justifyItems: 'stretch',
      autoFlow: 'row',
    },
    defaultStyles: {},
  },
  // ShadCN-style Card: opinionated wrapper that auto-centres at a sensible
  // max width with the theme's widget tokens (background, radius, border,
  // shadow, padding). Drops three named slots so the author always gets the
  // header / content / footer split for free — the slots themselves are not
  // exposed in the library and only live as children of a card.
  {
    type: 'card',
    label: 'Card',
    icon: <CreditCard size={14} />,
    isContainer: true,
    allowedChildren: ['card-header', 'card-content', 'card-footer'],
    defaultProps: {
      maxWidth: '440px',
      // `auto` margins centre the card horizontally in its parent regardless
      // of whether the parent uses block, flex, or grid layout.
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'card-header',
    label: 'Card header',
    icon: <PanelTop size={14} />,
    isContainer: true,
    allowedChildren: CARD_SLOT_CHILDREN,
    defaultProps: {
      textAlign: 'center',
      gap: '6px',
    },
    defaultStyles: {},
  },
  {
    type: 'card-content',
    label: 'Card content',
    icon: <LayoutTemplate size={14} />,
    isContainer: true,
    allowedChildren: CARD_SLOT_CHILDREN,
    defaultProps: {
      gap: '12px',
    },
    defaultStyles: {},
  },
  {
    type: 'card-footer',
    label: 'Card footer',
    icon: <PanelBottom size={14} />,
    isContainer: true,
    allowedChildren: CARD_SLOT_CHILDREN,
    defaultProps: {
      direction: 'row',
      justifyContent: 'flex-end',
      gap: '8px',
    },
    defaultStyles: {},
  },
  {
    type: 'heading',
    label: 'Heading',
    icon: <HeadingIcon size={14} />,
    hasContent: true,
    defaultProps: {
      level: '2',
      textAlign: 'center',
      fontWeight: '600',
    },
    defaultStyles: {},
  },
  {
    type: 'text',
    label: 'Text',
    icon: <Type size={14} />,
    hasContent: true,
    defaultProps: {
      textAlign: 'left',
      fontSize: 'var(--fk-font-base-size, 16px)',
      lineHeight: '1.5',
    },
    defaultStyles: {},
  },
  {
    type: 'image',
    label: 'Image',
    icon: <ImageIcon size={14} />,
    defaultProps: {
      src: '/logo_ferriskey.png',
      alt: 'Logo',
      width: '64px',
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'spacer',
    label: 'Spacer',
    icon: <MoveVertical size={14} />,
    defaultProps: { height: '16px' },
    defaultStyles: {},
  },
  {
    type: 'divider',
    label: 'Divider',
    icon: <Minus size={14} />,
    defaultProps: {
      color: 'var(--fk-color-body-text, #d1d5db)',
      thickness: '1px',
      width: '100%',
    },
    defaultStyles: {},
  },
  {
    type: 'button',
    label: 'Button',
    icon: <MousePointerClick size={14} />,
    hasContent: true,
    defaultProps: {
      variant: 'primary',
      href: '#',
      fullWidth: 'true',
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'input',
    label: 'Input',
    icon: <TextCursorInput size={14} />,
    defaultProps: {
      label: 'Username',
      placeholder: '',
      type: 'text',
    },
    defaultStyles: {},
  },
  {
    type: 'page-content',
    label: 'Page content',
    icon: <LayoutTemplate size={14} />,
    defaultProps: {},
    defaultStyles: {},
  },
  {
    type: 'username_input',
    label: 'Username input',
    icon: <User size={14} />,
    defaultProps: {
      label: 'Username',
      placeholder: '',
      type: 'text',
      name: 'username',
    },
    defaultStyles: {},
  },
  {
    type: 'first_name_input',
    label: 'First name input',
    icon: <IdCard size={14} />,
    defaultProps: {
      label: 'First name',
      placeholder: '',
      type: 'text',
      name: 'first_name',
    },
    defaultStyles: {},
  },
  {
    type: 'last_name_input',
    label: 'Last name input',
    icon: <IdCard size={14} />,
    defaultProps: {
      label: 'Last name',
      placeholder: '',
      type: 'text',
      name: 'last_name',
    },
    defaultStyles: {},
  },
  {
    type: 'email_input',
    label: 'Email input',
    icon: <AtSign size={14} />,
    defaultProps: {
      label: 'Email',
      placeholder: 'you@example.com',
      type: 'text',
      name: 'email',
    },
    defaultStyles: {},
  },
  {
    type: 'password_input',
    label: 'Password input',
    icon: <LockKeyhole size={14} />,
    defaultProps: {
      label: 'Password',
      placeholder: '',
      type: 'password',
      name: 'password',
    },
    defaultStyles: {},
  },
  {
    type: 'password_confirm_input',
    label: 'Confirm password',
    icon: <ShieldCheck size={14} />,
    defaultProps: {
      label: 'Confirm password',
      placeholder: '',
      type: 'password',
      name: 'password_confirm',
    },
    defaultStyles: {},
  },
  {
    type: 'totp_input',
    label: 'TOTP input',
    icon: <KeyRound size={14} />,
    defaultProps: {
      label: 'One-time code',
      placeholder: '123 456',
      type: 'text',
      name: 'totp',
    },
    defaultStyles: {},
  },
  {
    type: 'submit_button',
    label: 'Submit button',
    icon: <CheckSquare size={14} />,
    hasContent: true,
    defaultProps: {
      variant: 'primary',
      fullWidth: 'true',
      submit: 'true',
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'identity_providers',
    label: 'Identity providers',
    icon: <Globe size={14} />,
    defaultProps: {
      // Separator label displayed above the list ("Or continue with").
      separatorLabel: 'Or continue with',
      // Localizable button label prefix; the provider's display name is appended.
      buttonLabel: 'Continue with',
    },
    defaultStyles: {},
  },
  {
    type: 'magic_link_button',
    label: 'Magic link',
    icon: <Mail size={14} />,
    hasContent: true,
    defaultProps: {
      // Outline by default — magic link is an alternative auth path, not the
      // primary CTA (which is the submit button).
      variant: 'outline',
      fullWidth: 'true',
      // Centred icon+label group by default. Switch to `left` for a
      // social-style alignment (icon on left, label flowing right).
      align: 'center',
    },
    defaultStyles: {},
  },
  {
    type: 'passkey_button',
    label: 'Passkey',
    icon: <Fingerprint size={14} />,
    hasContent: true,
    defaultProps: {
      variant: 'outline',
      fullWidth: 'true',
      align: 'center',
    },
    defaultStyles: {},
  },
  // Inline link that navigates to the realm's `/forgot-password` page. The
  // URL is relative (`../forgot-password`) so it works across realms without
  // the renderer needing to know the realm name at render time. Click during
  // the builder preview is inert (renderer flips `runtime: false`); at
  // runtime the browser handles the navigation natively.
  {
    type: 'forgot_password_link',
    label: 'Forgot password',
    icon: <HelpCircle size={14} />,
    hasContent: true,
    defaultProps: {
      textAlign: 'center',
    },
    defaultStyles: {},
  },
  // Inline "Back to login" link. Used on Register / Forgot password / Magic
  // link request / Verify email / Reset password / Magic link verify pages —
  // anywhere the user can reasonably bail out and try the normal sign-in
  // flow instead. Same relative-href trick as `forgot_password_link`.
  {
    type: 'back_to_login_link',
    label: 'Back to login',
    icon: <ArrowLeft size={14} />,
    hasContent: true,
    defaultProps: {
      textAlign: 'center',
    },
    defaultStyles: {},
  },
  // Inline "Don't have an account? Sign up" link. Mirror of back_to_login —
  // sits on the Login page (and anywhere else the user might want to bail
  // out and create an account instead). Routes to `./register`.
  {
    type: 'register_link',
    label: 'Register link',
    icon: <UserPlus size={14} />,
    hasContent: true,
    defaultProps: {
      textAlign: 'center',
    },
    defaultStyles: {},
  },
  // QR code rendered from the `otpauth://` URL supplied by the TOTP setup
  // endpoint. The renderer pulls the URL from a per-page render option
  // (`totpSetup.otpauthUrl`) — the admin doesn't author the value, only
  // controls the placement.
  {
    type: 'totp_qr_code',
    label: 'TOTP QR code',
    icon: <QrCode size={14} />,
    defaultProps: {
      // Side length in pixels for the rendered QR. Most authenticator
      // apps scan a 160–200px QR comfortably.
      size: '180',
      align: 'center',
    },
    defaultStyles: {},
  },
  // Plain-text fallback for the TOTP secret — typed by hand when scanning
  // isn't possible. Renders the secret string with a copy affordance.
  {
    type: 'totp_secret',
    label: 'TOTP secret',
    icon: <Hash size={14} />,
    defaultProps: {
      align: 'center',
    },
    defaultStyles: {},
  },
  // Inline error banner. Renders the latest submit failure message
  // surfaced by the page's submit handler (invalid credentials, etc.) —
  // analogous to the red banner in the React default theme. Auto-hides
  // when there's no error; the canvas preview shows a placeholder so
  // admins can style the banner without triggering an actual failure.
  {
    type: 'form_error_banner',
    label: 'Form error',
    icon: <AlertCircle size={14} />,
    defaultProps: {
      variant: 'destructive',
    },
    defaultStyles: {},
  },
]

/**
 * Block types that are *only ever* required by a specific portal page —
 * these stay out of the generic palette section and only surface in the
 * "Required for this page" group when the backend requests them. Optional
 * identity-related inputs (first/last name, username, password-confirm)
 * are deliberately NOT in this set: they're genuinely optional building
 * blocks for forms like Register, so they should appear in the regular
 * library palette and be hideable per page via `HIDDEN_BLOCKS_BY_PAGE_TYPE`.
 */
export const REQUIRED_BLOCK_TYPES = new Set([
  'email_input',
  'password_input',
  'totp_input',
  'submit_button',
  'identity_providers',
])

/**
 * Block types hidden from the component library. Either layout-tree only
 * (`page-content`), or named slots that exist solely as children of a parent
 * block (`card-header` / `card-content` / `card-footer` — pre-populated by
 * the Card's `getDefaultNode` and never authored standalone).
 */
export const LAYOUT_ONLY_BLOCK_TYPES = new Set([
  'page-content',
  'card-header',
  'card-content',
  'card-footer',
])

/**
 * Per-page exclusion list: block types that don't make semantic sense on a
 * given portal page and shouldn't be offered in the library while editing
 * that page (e.g., no "Forgot password" link on the Register page — the
 * user doesn't have an account yet). The library filters its generic and
 * required sections against this set so the palette stays focused on what
 * the admin can actually use.
 *
 * Lookup is by `Schemas.PortalPageType` string. Pages not listed get the
 * full palette.
 */
export const HIDDEN_BLOCKS_BY_PAGE_TYPE: Record<string, ReadonlySet<string>> = {
  login: new Set([
    // The "back to login" link is meaningless when we're already there.
    'back_to_login_link',
  ]),
  register: new Set([
    // Auth alternatives only make sense once the user has an account.
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'register_link',
    // Identity providers can stay (social registration is valid).
    // TOTP makes no sense before the account exists.
    'totp_input',
  ]),
  forgot_password: new Set([
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'password_input',
    'password_confirm_input',
    'totp_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
  reset_password: new Set([
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'email_input',
    'totp_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
  totp: new Set([
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'register_link',
    'email_input',
    'password_input',
    'password_confirm_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
  magic_link_request: new Set([
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'password_input',
    'password_confirm_input',
    'totp_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
  magic_link_verify: new Set([
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'register_link',
    'email_input',
    'password_input',
    'password_confirm_input',
    'totp_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
  verify_email: new Set([
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'email_input',
    'password_input',
    'password_confirm_input',
    'totp_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
  email_verified: new Set([
    // Pure confirmation screen — no auth alternatives, no form fields.
    // Just heading + text + the back-to-login link (or a button styled
    // as a CTA).
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'email_input',
    'password_input',
    'password_confirm_input',
    'totp_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
    'register_link',
  ]),
  totp_setup: new Set([
    // QR + code entry only. No auth alternatives (the user is already
    // mid-flow) and no other identity fields.
    'magic_link_button',
    'passkey_button',
    'forgot_password_link',
    'register_link',
    'email_input',
    'password_input',
    'password_confirm_input',
    'first_name_input',
    'last_name_input',
    'username_input',
    'identity_providers',
  ]),
}

/**
 * Inverse of `HIDDEN_BLOCKS_BY_PAGE_TYPE`: blocks that only make sense on
 * a specific portal page. The library hides them everywhere else so the
 * admin doesn't drag in a `totp_qr_code` on the login page and end up
 * with a placeholder that never resolves a real URL.
 *
 * Keyed by block type, value is the allow-list of page types where it
 * should appear.
 */
export const RESTRICTED_TO_PAGE_TYPE: Record<string, ReadonlySet<string>> = {
  totp_qr_code: new Set(['totp_setup']),
  totp_secret: new Set(['totp_setup']),
}

const DEFAULT_CONTENT: Partial<Record<string, string>> = {
  heading: 'Welcome',
  text: 'Sign in to your account.',
  button: 'Continue',
  submit_button: 'Continue',
  magic_link_button: 'Sign in with a magic link',
  passkey_button: 'Sign in with a passkey',
  forgot_password_link: 'Forgot password?',
  back_to_login_link: 'Back to login',
  register_link: 'Don\u2019t have an account? Sign up',
}

/**
 * Build a fully-formed `BuilderNode` (including `id`) of the given type using
 * the same defaults as `getDefaultNode`. Used by `getDefaultNode('card')` to
 * pre-populate the header/content/footer slots so the author drops a Card and
 * gets a working layout immediately — no need to add the three slots by hand.
 */
function buildNode(type: string): BuilderNode {
  const seed = getDefaultNode(type)
  return { ...seed, id: generateNodeId() }
}

export function getDefaultNode(type: string): Omit<BuilderNode, 'id'> {
  const def = portalComponents.find((c) => c.type === type)
  const base: Omit<BuilderNode, 'id'> = {
    type,
    props: { ...(def?.defaultProps ?? {}) },
    styles: { ...(def?.defaultStyles ?? {}) },
    children: [],
    content: def?.hasContent ? (DEFAULT_CONTENT[type] ?? '') : undefined,
  }

  // A Card is unusable without its three slots — pre-fill them so dropping a
  // Card produces the same shape ShadCN users expect (header / content /
  // footer all present, ready for blocks). The slot defaults are produced by
  // recursing through `buildNode`, which gives each one a stable id.
  if (type === 'card') {
    return { ...base, children: [buildNode('card-header'), buildNode('card-content'), buildNode('card-footer')] }
  }

  return base
}
