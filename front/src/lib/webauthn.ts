export function isWebAuthnAvailable(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof window.PublicKeyCredential !== 'undefined'
  )
}

export async function isConditionalMediationAvailable(): Promise<boolean> {
  if (!isWebAuthnAvailable()) return false
  if (typeof PublicKeyCredential.isConditionalMediationAvailable !== 'function') return false
  return PublicKeyCredential.isConditionalMediationAvailable()
}

function base64UrlToBuffer(base64url: string): ArrayBuffer {
  const base64 = base64url.replace(/-/g, '+').replace(/_/g, '/')
  const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), '=')
  const binary = atob(padded)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes.buffer
}

function bufferToBase64Url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  for (const byte of bytes) {
    binary += String.fromCharCode(byte)
  }
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

export interface PublicKeyCredentialRequestOptionsJSON {
  challenge: string
  timeout?: number
  rpId?: string
  allowCredentials?: Array<{
    type: string
    id: string
    transports?: string[]
  }>
  userVerification?: string
  extensions?: Record<string, unknown>
}

function toPublicKeyRequestOptions(
  options: PublicKeyCredentialRequestOptionsJSON
): PublicKeyCredentialRequestOptions {
  return {
    challenge: base64UrlToBuffer(options.challenge),
    timeout: options.timeout,
    rpId: options.rpId,
    userVerification: (options.userVerification as UserVerificationRequirement) ?? 'preferred',
    allowCredentials: options.allowCredentials?.map((cred) => ({
      type: cred.type as PublicKeyCredentialType,
      id: base64UrlToBuffer(cred.id),
      transports: cred.transports as AuthenticatorTransport[] | undefined,
    })),
  }
}

function serializeAssertionCredential(
  credential: PublicKeyCredential & { response: AuthenticatorAssertionResponse }
): Record<string, unknown> {
  const response = credential.response

  const result: Record<string, unknown> = {
    id: credential.id,
    rawId: bufferToBase64Url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToBase64Url(response.authenticatorData),
      clientDataJSON: bufferToBase64Url(response.clientDataJSON),
      signature: bufferToBase64Url(response.signature),
      userHandle: response.userHandle ? bufferToBase64Url(response.userHandle) : null,
    },
  }

  if (credential.authenticatorAttachment) {
    result.authenticatorAttachment = credential.authenticatorAttachment
  }

  return result
}

export async function startAuthentication(
  options: PublicKeyCredentialRequestOptionsJSON
): Promise<Record<string, unknown>> {
  const credential = (await navigator.credentials.get({
    publicKey: toPublicKeyRequestOptions(options),
  })) as PublicKeyCredential & { response: AuthenticatorAssertionResponse }

  if (!credential) {
    throw new Error('Authentication was cancelled or failed')
  }

  return serializeAssertionCredential(credential)
}

/**
 * Start conditional mediation (autofill UI).
 * The browser will show passkeys in the username field's autocomplete dropdown.
 * Returns the assertion when the user selects a passkey, or null if aborted.
 */
export async function startConditionalAuthentication(
  options: PublicKeyCredentialRequestOptionsJSON,
  signal?: AbortSignal
): Promise<Record<string, unknown> | null> {
  try {
    const credential = (await navigator.credentials.get({
      publicKey: toPublicKeyRequestOptions(options),
      mediation: 'conditional' as CredentialMediationRequirement,
      signal,
    })) as PublicKeyCredential & { response: AuthenticatorAssertionResponse }

    if (!credential) return null

    return serializeAssertionCredential(credential)
  } catch {
    return null
  }
}

export async function startRegistration(
  options: Record<string, unknown>
): Promise<Record<string, unknown>> {
  const publicKey = options as unknown as {
    challenge: string
    rp: { name: string; id?: string }
    user: { id: string; name: string; displayName: string }
    pubKeyCredParams: Array<{ type: string; alg: number }>
    timeout?: number
    excludeCredentials?: Array<{ type: string; id: string; transports?: string[] }>
    authenticatorSelection?: {
      authenticatorAttachment?: string
      residentKey?: string
      requireResidentKey?: boolean
      userVerification?: string
    }
    attestation?: string
  }

  const creationOptions: PublicKeyCredentialCreationOptions = {
    challenge: base64UrlToBuffer(publicKey.challenge),
    rp: publicKey.rp,
    user: {
      id: base64UrlToBuffer(publicKey.user.id),
      name: publicKey.user.name,
      displayName: publicKey.user.displayName,
    },
    pubKeyCredParams: publicKey.pubKeyCredParams.map((p) => ({
      type: p.type as PublicKeyCredentialType,
      alg: p.alg,
    })),
    timeout: publicKey.timeout,
    excludeCredentials: publicKey.excludeCredentials?.map((cred) => ({
      type: cred.type as PublicKeyCredentialType,
      id: base64UrlToBuffer(cred.id),
      transports: cred.transports as AuthenticatorTransport[] | undefined,
    })),
    authenticatorSelection: {
      authenticatorAttachment: publicKey.authenticatorSelection
        ?.authenticatorAttachment as AuthenticatorAttachment | undefined,
      residentKey: 'required' as ResidentKeyRequirement,
      requireResidentKey: true,
      userVerification: (publicKey.authenticatorSelection
        ?.userVerification as UserVerificationRequirement) ?? 'required',
    },
    attestation: (publicKey.attestation as AttestationConveyancePreference) ?? 'none',
  }

  const credential = (await navigator.credentials.create({
    publicKey: creationOptions,
  })) as PublicKeyCredential & { response: AuthenticatorAttestationResponse }

  if (!credential) {
    throw new Error('Registration was cancelled or failed')
  }

  const response = credential.response

  return {
    id: credential.id,
    rawId: bufferToBase64Url(credential.rawId),
    type: credential.type,
    response: {
      attestationObject: bufferToBase64Url(response.attestationObject),
      clientDataJSON: bufferToBase64Url(response.clientDataJSON),
    },
  }
}
