import type { BuilderNode } from '@/lib/builder-core'

export interface EmailTemplatePreset {
  id: string
  name: string
  emailType: string
  description: string
  tree: BuilderNode[]
}

function section(
  id: string,
  props: Record<string, unknown>,
  children: BuilderNode[],
): BuilderNode {
  return { id, type: 'mj-section', props, styles: {}, children }
}

function column(id: string, children: BuilderNode[]): BuilderNode {
  return { id, type: 'mj-column', props: {}, styles: {}, children }
}

function text(
  id: string,
  content: string,
  props: Record<string, unknown> = {},
): BuilderNode {
  return {
    id,
    type: 'mj-text',
    props: { 'font-size': '14px', color: '#333333', 'line-height': '1.6', ...props },
    styles: {},
    children: [],
    content,
  }
}

function button(
  id: string,
  content: string,
  props: Record<string, unknown> = {},
): BuilderNode {
  return {
    id,
    type: 'mj-button',
    props: {
      'background-color': '#4F46E5',
      color: '#ffffff',
      'font-size': '16px',
      'border-radius': '6px',
      'inner-padding': '12px 32px',
      href: '#',
      ...props,
    },
    styles: {},
    children: [],
    content,
  }
}

function divider(id: string): BuilderNode {
  return {
    id,
    type: 'mj-divider',
    props: { 'border-color': '#e5e7eb', 'border-width': '1px' },
    styles: {},
    children: [],
  }
}

export const emailTemplatePresets: EmailTemplatePreset[] = [
  {
    id: 'preset-reset-password',
    name: 'Reset Password',
    emailType: 'reset_password',
    description: 'Password reset email with a CTA button and expiration notice.',
    tree: [
      section(
        'preset-rp-header',
        { 'background-color': '#4F46E5', padding: '32px 0' },
        [
          column('preset-rp-header-col', [
            text('preset-rp-header-title', '<h1 style="margin:0;color:#ffffff;font-size:24px;text-align:center;">Reset Your Password</h1>', {
              color: '#ffffff',
              align: 'center',
            }),
          ]),
        ],
      ),
      section(
        'preset-rp-body',
        { 'background-color': '#ffffff', padding: '32px 24px' },
        [
          column('preset-rp-body-col', [
            text(
              'preset-rp-body-greeting',
              '<p style="margin:0;">Hi {{user.first_name}},</p>',
            ),
            text(
              'preset-rp-body-message',
              '<p style="margin:0;">We received a request to reset your password. Click the button below to choose a new one.</p>',
            ),
            button('preset-rp-body-cta', 'Reset Password', {
              href: '{{reset_link}}',
              align: 'center',
            }),
            text(
              'preset-rp-body-alt',
              '<p style="margin:0;font-size:12px;color:#6b7280;">If the button doesn\'t work, copy and paste this link into your browser: {{reset_link}}</p>',
              { 'font-size': '12px', color: '#6b7280' },
            ),
          ]),
        ],
      ),
      section(
        'preset-rp-footer',
        { 'background-color': '#f9fafb', padding: '24px' },
        [
          column('preset-rp-footer-col', [
            divider('preset-rp-footer-divider'),
            text(
              'preset-rp-footer-expiry',
              '<p style="margin:0;text-align:center;font-size:12px;color:#9ca3af;">This link expires in {{expiration}}. If you did not request a password reset, you can safely ignore this email.</p>',
              { 'font-size': '12px', color: '#9ca3af', align: 'center' },
            ),
          ]),
        ],
      ),
    ],
  },
  {
    id: 'preset-magic-link',
    name: 'Magic Link',
    emailType: 'magic_link',
    description: 'Passwordless sign-in email with a magic link button.',
    tree: [
      section(
        'preset-ml-header',
        { 'background-color': '#4F46E5', padding: '32px 0' },
        [
          column('preset-ml-header-col', [
            text('preset-ml-header-title', '<h1 style="margin:0;color:#ffffff;font-size:24px;text-align:center;">Sign In</h1>', {
              color: '#ffffff',
              align: 'center',
            }),
          ]),
        ],
      ),
      section(
        'preset-ml-body',
        { 'background-color': '#ffffff', padding: '32px 24px' },
        [
          column('preset-ml-body-col', [
            text(
              'preset-ml-body-greeting',
              '<p style="margin:0;">Hi {{user.first_name}},</p>',
            ),
            text(
              'preset-ml-body-message',
              '<p style="margin:0;">Click the button below to securely sign in to your account. No password needed.</p>',
            ),
            button('preset-ml-body-cta', 'Sign In', {
              href: '{{magic_link}}',
              align: 'center',
            }),
            text(
              'preset-ml-body-alt',
              '<p style="margin:0;font-size:12px;color:#6b7280;">If the button doesn\'t work, copy and paste this link into your browser: {{magic_link}}</p>',
              { 'font-size': '12px', color: '#6b7280' },
            ),
          ]),
        ],
      ),
      section(
        'preset-ml-footer',
        { 'background-color': '#f9fafb', padding: '24px' },
        [
          column('preset-ml-footer-col', [
            divider('preset-ml-footer-divider'),
            text(
              'preset-ml-footer-expiry',
              '<p style="margin:0;text-align:center;font-size:12px;color:#9ca3af;">This link expires in {{expiration}}. If you did not request this, you can safely ignore this email — no one else can use this link.</p>',
              { 'font-size': '12px', color: '#9ca3af', align: 'center' },
            ),
          ]),
        ],
      ),
    ],
  },
  {
    id: 'preset-email-verification',
    name: 'Email Verification',
    emailType: 'email_verification',
    description: 'Email verification with a confirmation button.',
    tree: [
      section(
        'preset-ev-header',
        { 'background-color': '#4F46E5', padding: '32px 0' },
        [
          column('preset-ev-header-col', [
            text('preset-ev-header-title', '<h1 style="margin:0;color:#ffffff;font-size:24px;text-align:center;">Verify Your Email</h1>', {
              color: '#ffffff',
              align: 'center',
            }),
          ]),
        ],
      ),
      section(
        'preset-ev-body',
        { 'background-color': '#ffffff', padding: '32px 24px' },
        [
          column('preset-ev-body-col', [
            text(
              'preset-ev-body-greeting',
              '<p style="margin:0;">Hi {{user.first_name}},</p>',
            ),
            text(
              'preset-ev-body-message',
              '<p style="margin:0;">Thanks for signing up! Please verify your email address by clicking the button below.</p>',
            ),
            button('preset-ev-body-cta', 'Verify Email', {
              href: '{{verification_link}}',
              align: 'center',
            }),
            text(
              'preset-ev-body-alt',
              '<p style="margin:0;font-size:12px;color:#6b7280;">If the button doesn\'t work, copy and paste this link into your browser: {{verification_link}}</p>',
              { 'font-size': '12px', color: '#6b7280' },
            ),
          ]),
        ],
      ),
      section(
        'preset-ev-footer',
        { 'background-color': '#f9fafb', padding: '24px' },
        [
          column('preset-ev-footer-col', [
            divider('preset-ev-footer-divider'),
            text(
              'preset-ev-footer-expiry',
              '<p style="margin:0;text-align:center;font-size:12px;color:#9ca3af;">This link expires in {{expiration}}. If you did not create an account, please ignore this email.</p>',
              { 'font-size': '12px', color: '#9ca3af', align: 'center' },
            ),
          ]),
        ],
      ),
    ],
  },
]
