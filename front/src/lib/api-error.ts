import enErrors from '@/locales/en/errors.yaml'
import i18n, { preloadNamespaces, translate } from '@/lib/i18n'

export const ERRORS_NAMESPACE = 'errors'

const LAST_RESORT_KEY = 'common:toast.unexpected_error'

const FIELD_ERROR_SEPARATOR = ' — '

void preloadNamespaces(ERRORS_NAMESPACE).catch(() => undefined)

export interface ApiErrorPayload {
  code?: string
  status?: number
  reason?: string
  message?: string
  error?: string
  error_description?: string
  errors?: unknown
}

export interface FieldValidationError {
  field: string
  code: string
  message: string
}

export interface ApiRequestError extends Error {
  status?: number
  data?: Record<string, unknown>
}

interface RawValidationEntry {
  field: string
  code?: string
  message: string
}

function catalogMessage(code: unknown): string | undefined {
  if (typeof code !== 'string' || code.length === 0) return undefined

  const bundled = enErrors[code]
  if (typeof bundled !== 'string') return undefined

  const key = `${ERRORS_NAMESPACE}:${code}`

  if (i18n.exists(key)) {
    const translated = translate(key)
    if (typeof translated === 'string' && translated.length > 0 && translated !== key) {
      return translated
    }
  }

  return bundled
}

export function apiErrorPayload(error: unknown): ApiErrorPayload | undefined {
  if (!error || typeof error !== 'object') return undefined

  const data = (error as ApiRequestError).data
  if (data && typeof data === 'object') return data as ApiErrorPayload

  if (error instanceof Error) return undefined

  return error as ApiErrorPayload
}

function rawValidationEntries(body: unknown): RawValidationEntry[] {
  const entries = (body as { errors?: unknown } | null | undefined)?.errors
  if (!Array.isArray(entries)) return []

  return entries.filter(
    (entry): entry is RawValidationEntry =>
      typeof entry === 'object' &&
      entry !== null &&
      typeof (entry as RawValidationEntry).field === 'string' &&
      typeof (entry as RawValidationEntry).message === 'string'
  )
}

export function validationErrorsFromBody(body: unknown): FieldValidationError[] {
  return rawValidationEntries(body).map((entry) => ({
    field: entry.field,
    code: typeof entry.code === 'string' ? entry.code : '',
    message: catalogMessage(entry.code) ?? entry.message,
  }))
}

export function validationErrorsFrom(error: unknown): FieldValidationError[] {
  return validationErrorsFromBody(apiErrorPayload(error))
}

export interface PartitionedFieldErrors<Field extends string> {
  byField: Map<Field, string>
  unattached: string[]
}

export function partitionFieldErrors<Field extends string>(
  errors: readonly FieldValidationError[],
  resolveField: (field: string) => Field | undefined
): PartitionedFieldErrors<Field> {
  const collected = new Map<Field, string[]>()
  const unattached: string[] = []

  for (const error of errors) {
    const field = resolveField(error.field)

    if (field === undefined) {
      if (!unattached.includes(error.message)) unattached.push(error.message)
      continue
    }

    const messages = collected.get(field) ?? []
    if (!messages.includes(error.message)) messages.push(error.message)
    collected.set(field, messages)
  }

  return {
    byField: new Map(
      [...collected].map(([field, messages]) => [field, messages.join(FIELD_ERROR_SEPARATOR)])
    ),
    unattached,
  }
}

export function serverMessageFromBody(body: unknown): string | undefined {
  if (!body || typeof body !== 'object') return undefined

  const { message, error_description: errorDescription } = body as ApiErrorPayload

  if (typeof message === 'string' && message.length > 0) return message
  if (typeof errorDescription === 'string' && errorDescription.length > 0) return errorDescription

  return undefined
}

export function errorMessageFromBody(body: unknown): string | undefined {
  const direct = serverMessageFromBody(body)
  if (direct) return direct

  const messages = rawValidationEntries(body).map((entry) => entry.message)
  return messages.length > 0 ? messages.join(FIELD_ERROR_SEPARATOR) : undefined
}

export function apiErrorReason(error: unknown): string | undefined {
  const payload = apiErrorPayload(error)
  const reason = payload?.reason ?? payload?.error
  return typeof reason === 'string' && reason.length > 0 ? reason : undefined
}

export function apiErrorMessage(error: unknown, fallback: string = translate(LAST_RESORT_KEY)): string {
  const payload = apiErrorPayload(error)

  const fieldErrors = validationErrorsFromBody(payload)
  if (fieldErrors.length > 0) {
    const { byField, unattached } = partitionFieldErrors(fieldErrors, (field) => field)
    return [...byField.values(), ...unattached].join(FIELD_ERROR_SEPARATOR)
  }

  const fromCatalog = catalogMessage(payload?.reason) ?? catalogMessage(payload?.error)
  if (fromCatalog) return fromCatalog

  const serverMessage = serverMessageFromBody(payload)
  if (serverMessage) return serverMessage

  if (error instanceof Error && error.message.length > 0) return error.message

  return fallback
}
