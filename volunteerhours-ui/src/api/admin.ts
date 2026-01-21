import { requestJson } from './client'
import { requestMultipart } from './client'

export async function listCompetitions(): Promise<unknown[]> {
  return requestJson('/admin/competitions', { method: 'GET' })
}

export async function createCompetition(name: string): Promise<unknown> {
  return requestJson('/admin/competitions', {
    method: 'POST',
    body: JSON.stringify({ name }),
  })
}

export async function importCompetitions(file: File): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart('/admin/competitions/import', form)
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

export async function importVolunteerRecords(file: File): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart('/admin/records/volunteer/import', form)
}

export async function importContestRecords(file: File): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart('/admin/records/contest/import', form)
}
