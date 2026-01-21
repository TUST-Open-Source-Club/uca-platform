import { requestJson } from './client'

export async function listCompetitions(): Promise<unknown[]> {
  return requestJson('/admin/competitions', { method: 'GET' })
}

export async function createCompetition(name: string): Promise<unknown> {
  return requestJson('/admin/competitions', {
    method: 'POST',
    body: JSON.stringify({ name }),
  })
}

export async function listFormFields(): Promise<unknown[]> {
  return requestJson('/admin/form-fields', { method: 'GET' })
}

export async function createFormField(payload: Record<string, unknown>): Promise<unknown> {
  return requestJson('/admin/form-fields', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}
