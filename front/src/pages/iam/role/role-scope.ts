export const roleScopeLabelKey = (isClientRole: boolean) =>
  isClientRole ? 'role_scope.client' : 'role_scope.realm'

export const roleScopeHintKey = (isClientRole: boolean) =>
  isClientRole ? 'form.scope.hint.client' : 'form.scope.hint.realm'
