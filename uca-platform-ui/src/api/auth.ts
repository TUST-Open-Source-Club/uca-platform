import { requestJson } from './client'

export type PasskeyStartResponse = {
  session_id: string
  public_key: unknown
}

export type CurrentUser = {
  id: string
  username: string
  display_name: string
  role: 'student' | 'reviewer' | 'teacher' | 'admin'
}

export async function passkeyStart(username: string): Promise<PasskeyStartResponse> {
  return requestJson('/auth/passkey/login/start', {
    method: 'POST',
    body: JSON.stringify({ username }),
  })
}

export async function passkeyRegisterStart(username: string): Promise<PasskeyStartResponse> {
  return requestJson('/auth/passkey/register/start', {
    method: 'POST',
    body: JSON.stringify({ username }),
  })
}

export async function passkeyRegisterFinish(
  session_id: string,
  credential: Record<string, unknown>,
): Promise<{ passkey_id: string }> {
  return requestJson('/auth/passkey/register/finish', {
    method: 'POST',
    body: JSON.stringify({ session_id, credential }),
  })
}

export async function passkeyFinish(
  session_id: string,
  credential: Record<string, unknown>,
): Promise<{ user_id: string }> {
  return requestJson('/auth/passkey/login/finish', {
    method: 'POST',
    body: JSON.stringify({ session_id, credential }),
  })
}

export async function totpVerify(username: string, code: string): Promise<{ user_id: string }> {
  return requestJson('/auth/totp/verify', {
    method: 'POST',
    body: JSON.stringify({ username, code }),
  })
}

export async function recoveryVerify(username: string, code: string): Promise<{ user_id: string }> {
  return requestJson('/auth/recovery/verify', {
    method: 'POST',
    body: JSON.stringify({ username, code }),
  })
}

export async function passwordLogin(username: string, password: string): Promise<{ user_id: string }> {
  return requestJson('/auth/password/login', {
    method: 'POST',
    body: JSON.stringify({ username, password }),
  })
}

export async function loginOptions(username: string): Promise<{ methods: string[] }> {
  const query = new URLSearchParams({ username }).toString()
  return requestJson(`/auth/login/options?${query}`, { method: 'GET' })
}

export async function listDevices(): Promise<unknown[]> {
  return requestJson('/auth/devices', { method: 'GET' })
}

export async function getCurrentUser(): Promise<CurrentUser> {
  return requestJson('/auth/me', { method: 'GET' })
}

export async function regenerateRecoveryCodes(): Promise<{ codes: string[] }> {
  return requestJson('/auth/recovery/regenerate', { method: 'POST' })
}

export async function bootstrapStatus(): Promise<{ ready: boolean; needs_totp: boolean }> {
  return requestJson('/auth/bootstrap/status', { method: 'GET' })
}

export async function bootstrapAdmin(payload: {
  username: string
  display_name: string
  token?: string
}): Promise<{ user_id: string }> {
  return requestJson('/auth/bootstrap', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function bindEmail(email: string): Promise<{ status: string }> {
  return requestJson('/auth/email/bind', {
    method: 'POST',
    body: JSON.stringify({ email }),
  })
}

export async function changePassword(payload: {
  current_password: string
  new_password: string
}): Promise<{ status: string }> {
  return requestJson('/auth/password/change', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function passwordResetRequest(username: string): Promise<{ status: string }> {
  return requestJson('/auth/password/reset/request', {
    method: 'POST',
    body: JSON.stringify({ username }),
  })
}

export async function passwordResetConfirm(payload: {
  token: string
  new_password: string
}): Promise<{ status: string }> {
  return requestJson('/auth/password/reset/confirm', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function inviteStatus(token: string): Promise<{
  valid: boolean
  email?: string
  username?: string
  display_name?: string
  role?: string
}> {
  const query = new URLSearchParams({ token }).toString()
  return requestJson(`/auth/invite/status?${query}`, { method: 'GET' })
}

export async function inviteAccept(token: string): Promise<{ user_id: string; username: string; role: string }> {
  return requestJson('/auth/invite/accept', {
    method: 'POST',
    body: JSON.stringify({ token }),
  })
}

export async function resetStatus(token: string): Promise<{ valid: boolean; purpose?: string }> {
  const query = new URLSearchParams({ token }).toString()
  return requestJson(`/auth/reset/status?${query}`, { method: 'GET' })
}

export async function resetConsume(token: string): Promise<{ user_id: string; purpose: string }> {
  return requestJson('/auth/reset/consume', {
    method: 'POST',
    body: JSON.stringify({ token }),
  })
}

export async function totpEnrollStart(payload?: {
  device_label?: string
}): Promise<{ enrollment_id: string; otpauth_url: string }> {
  return requestJson('/auth/totp/enroll/start', {
    method: 'POST',
    body: JSON.stringify(payload ?? {}),
  })
}

export async function totpEnrollFinish(payload: {
  enrollment_id: string
  code: string
}): Promise<{ status: string }> {
  return requestJson('/auth/totp/enroll/finish', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}
