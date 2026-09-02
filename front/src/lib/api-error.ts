/**
 * One entry of a 422 body (`ValidationErrorResponse` on the API side):
 * `{ errors: [{ field, message }] }`. `field` names the request field the
 * message belongs to, which is what lets a form show it under the right
 * input instead of in a toast.
 */
export interface FieldValidationError {
  field: string
  message: string
}

/** The error our `fetcher` throws on a non-2xx response. */
export interface ApiRequestError extends Error {
  status?: number
  data?: Record<string, unknown>
}

/** Read the `errors` array out of a parsed error body, ignoring malformed entries. */
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

/**
 * Pull the per-field validation errors out of a thrown request error, so a
 * form can attach each message to the input that caused it.
 */
export function validationErrorsFrom(error: unknown): FieldValidationError[] {
  return validationErrorsFromBody((error as ApiRequestError | null | undefined)?.data)
}

/**
 * Turn a parsed error body into a sentence worth showing. The API speaks
 * three dialects — `{ message }`, RFC 6749's `{ error_description }`, and a
 * 422's `{ errors: [...] }` — and the fetcher used to read only the first,
 * so a rejected password surfaced as "HTTP 422: Unprocessable Entity"
 * (issue #1302).
 */
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
