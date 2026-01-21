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

export async function listDevices(): Promise<unknown[]> {
  return requestJson('/auth/devices', { method: 'GET' })
}

export async function getCurrentUser(): Promise<CurrentUser> {
  return requestJson('/auth/me', { method: 'GET' })
}

export async function regenerateRecoveryCodes(): Promise<{ codes: string[] }> {
  return requestJson('/auth/recovery/regenerate', { method: 'POST' })
}
