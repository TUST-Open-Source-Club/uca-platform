import { requestJson, requestMultipart } from './client'

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

export async function deleteStudent(studentNo: string): Promise<unknown> {
  return requestJson(`/admin/students/${encodeURIComponent(studentNo)}`, { method: 'DELETE' })
}

export async function deleteVolunteerRecord(recordId: string): Promise<unknown> {
  return requestJson(`/admin/records/volunteer/${recordId}`, { method: 'DELETE' })
}

export async function deleteContestRecord(recordId: string): Promise<unknown> {
  return requestJson(`/admin/records/contest/${recordId}`, { method: 'DELETE' })
}

export async function listDeletedStudents(): Promise<unknown[]> {
  return requestJson('/admin/deleted/students', { method: 'GET' })
}

export async function listDeletedVolunteerRecords(): Promise<unknown[]> {
  return requestJson('/admin/deleted/records/volunteer', { method: 'GET' })
}

export async function listDeletedContestRecords(): Promise<unknown[]> {
  return requestJson('/admin/deleted/records/contest', { method: 'GET' })
}

export async function purgeStudent(studentNo: string): Promise<unknown> {
  return requestJson(`/admin/purge/students/${encodeURIComponent(studentNo)}`, { method: 'DELETE' })
}

export async function purgeVolunteerRecord(recordId: string): Promise<unknown> {
  return requestJson(`/admin/purge/records/volunteer/${recordId}`, { method: 'DELETE' })
}

export async function purgeContestRecord(recordId: string): Promise<unknown> {
  return requestJson(`/admin/purge/records/contest/${recordId}`, { method: 'DELETE' })
}
