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

export type StudentUpdatePayload = {
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

export async function importStudents(
  file: File,
  fieldMap?: Record<string, string>,
  allowLogin?: boolean,
): Promise<{ inserted: number; updated: number }> {
  const form = new FormData()
  form.append('file', file)
  if (fieldMap && Object.keys(fieldMap).length) {
    form.append('field_map', JSON.stringify(fieldMap))
  }
  if (allowLogin !== undefined) {
    form.append('allow_login', allowLogin ? 'true' : 'false')
  }
  return requestMultipart('/students/import', form)
}

export async function updateStudent(
  studentNo: string,
  payload: StudentUpdatePayload,
): Promise<unknown> {
  return requestJson(`/students/${encodeURIComponent(studentNo)}`, {
    method: 'PUT',
    body: JSON.stringify(payload),
  })
}
