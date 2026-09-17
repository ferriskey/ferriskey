import { z } from 'zod'
import { translate } from '@/lib/i18n'
import { AUTH_NAMESPACE } from '../constants'

// RFC 8628 §6.1: 20-character base20 alphabet, vowels and visually ambiguous
// glyphs excluded so the code stays unambiguous when read aloud or typed on a
// constrained device. Must stay in sync with `USER_CODE_CHARSET` in the
// backend (`core/src/domain/authentication/device_flow/entities.rs`).
export const USER_CODE_CHARSET = 'BCDFGHJKLMNPQRSTVWXZ'

const USER_CODE_REGEX = new RegExp(`^[${USER_CODE_CHARSET}]{4}-[${USER_CODE_CHARSET}]{4}$`)

export const deviceVerifySchema = z.object({
  user_code: z
    .string()
    .trim()
    .transform((value) => value.toUpperCase())
    .pipe(
      z
        .string()
        .regex(USER_CODE_REGEX, {
          error: () => translate(`${AUTH_NAMESPACE}:validation.device_code_format`),
        })
    ),
})

export type DeviceVerifySchema = z.infer<typeof deviceVerifySchema>
