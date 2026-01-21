import { requestJson, requestMultipart } from './client'

export type StudentPayload = {
  student_no: string
  name: string
  gender: string
  department: string
  major: string
  class_name: string
  phone: string
}

export async function createStudent(payload: StudentPayload): Promise<unknown> {
  return requestJson('/students', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function queryStudents(filters: Record<string, unknown>): Promise<unknown[]> {
  return requestJson('/students/query', {
    method: 'POST',
    body: JSON.stringify(filters),
  })
}

export async function importStudents(file: File): Promise<{ inserted: number; updated: number }> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart('/students/import', form)
}
