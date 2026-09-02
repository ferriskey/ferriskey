export interface FieldValidationError {
  field: string
  message: string
}

export interface ApiRequestError extends Error {
  status?: number
  data?: Record<string, unknown>
}

export function validationErrorsFromBody(body: unknown): FieldValidationError[] {
  const errors = (body as { errors?: unknown } | null | undefined)?.errors
  if (!Array.isArray(errors)) return []
  return errors.filter(
    (entry): entry is FieldValidationError =>
      typeof entry === 'object' &&
      entry !== null &&
      typeof (entry as FieldValidationError).field === 'string' &&
      typeof (entry as FieldValidationError).message === 'string'
  )
}

export function validationErrorsFrom(error: unknown): FieldValidationError[] {
  return validationErrorsFromBody((error as ApiRequestError | null | undefined)?.data)
}

export function errorMessageFromBody(body: unknown): string | undefined {
  if (!body || typeof body !== 'object') return undefined
  const { message, error_description: errorDescription } = body as Record<string, unknown>

  if (typeof message === 'string' && message.length > 0) return message
  if (typeof errorDescription === 'string' && errorDescription.length > 0) {
    return errorDescription
  }

  const messages = validationErrorsFromBody(body).map((error) => error.message)
  return messages.length > 0 ? messages.join(' — ') : undefined
}

export function apiErrorMessage(error: unknown, fallback: string): string {
  const fieldErrors = validationErrorsFrom(error)

  if (fieldErrors.length > 0) {
    return fieldErrors.map((fieldError) => fieldError.message).join(' — ')
  }

  if (error instanceof Error && error.message.length > 0) {
    return error.message
  }

  return fallback
}
