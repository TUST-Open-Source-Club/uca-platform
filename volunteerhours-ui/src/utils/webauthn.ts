type PublicKeyRequestInput = {
  publicKey?: PublicKeyCredentialRequestOptions
} & PublicKeyCredentialRequestOptions

export function normalizeRequestOptions(input: PublicKeyRequestInput): PublicKeyCredentialRequestOptions {
  const options = (input.publicKey ?? input) as PublicKeyCredentialRequestOptions
  return {
    ...options,
    challenge: toArrayBuffer(options.challenge),
    allowCredentials: options.allowCredentials?.map((cred) => ({
      ...cred,
      id: toArrayBuffer(cred.id),
    })),
  }
}

export function credentialToJson(credential: PublicKeyCredential) {
  const response = credential.response as AuthenticatorAssertionResponse
  return {
    id: credential.id,
    rawId: bufferToBase64Url(credential.rawId),
    type: credential.type,
    response: {
      authenticatorData: bufferToBase64Url(response.authenticatorData),
      clientDataJSON: bufferToBase64Url(response.clientDataJSON),
      signature: bufferToBase64Url(response.signature),
      userHandle: response.userHandle ? bufferToBase64Url(response.userHandle) : null,
    },
    clientExtensionResults: credential.getClientExtensionResults(),
  }
}

function base64UrlToBuffer(value: string): ArrayBuffer {
  const padding = '='.repeat((4 - (value.length % 4)) % 4)
  const base64 = (value + padding).replace(/-/g, '+').replace(/_/g, '/')
  const raw = window.atob(base64)
  const buffer = new Uint8Array(raw.length)
  for (let i = 0; i < raw.length; i += 1) {
    buffer[i] = raw.charCodeAt(i)
  }
  return buffer.buffer
}

function toArrayBuffer(value: BufferSource | string): ArrayBuffer {
  if (typeof value === 'string') {
    return base64UrlToBuffer(value)
  }
  if (value instanceof ArrayBuffer) {
    return value
  }
  return value.buffer.slice(value.byteOffset, value.byteOffset + value.byteLength)
}

function bufferToBase64Url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer)
  let binary = ''
  bytes.forEach((byte) => {
    binary += String.fromCharCode(byte)
  })
  const base64 = window.btoa(binary)
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '')
}
