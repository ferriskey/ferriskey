interface FieldValidationError {
  field: string
  message: string
}

export function apiErrorMessage(error: unknown, fallback: string): string {
  const data = (error as { data?: { errors?: FieldValidationError[] } } | null)?.data
  const fieldErrors = data?.errors

  if (Array.isArray(fieldErrors) && fieldErrors.length > 0) {
    return fieldErrors.map((fieldError) => fieldError.message).join(' — ')
  }

  if (error instanceof Error && error.message.length > 0) {
    return error.message
  }

  return fallback
}
