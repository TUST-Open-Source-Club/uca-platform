import { requestJson, requestMultipart } from './client'

export type CompetitionItem = {
  id: string
  name: string
  year?: number | null
  category?: string | null
}

export async function listCompetitions(): Promise<CompetitionItem[]> {
  return requestJson('/admin/competitions', { method: 'GET' })
}

export async function createCompetition(payload: {
  name: string
  year?: number | null
  category?: string | null
}): Promise<CompetitionItem> {
  return requestJson('/admin/competitions', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function updateCompetition(
  competitionId: string,
  payload: {
    name: string
    year?: number | null
    category?: string | null
  },
): Promise<CompetitionItem> {
  return requestJson(`/admin/competitions/${competitionId}`, {
    method: 'PUT',
    body: JSON.stringify(payload),
  })
}

export async function deleteCompetition(competitionId: string): Promise<{ status: string }> {
  return requestJson(`/admin/competitions/${competitionId}`, { method: 'DELETE' })
}

export type CompetitionSheetPlan = {
  name: string
  year?: number | null
}

export async function importCompetitions(
  file: File,
  defaultYear?: number | null,
  sheetPlan?: CompetitionSheetPlan[],
): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  if (defaultYear !== undefined && defaultYear !== null) {
    form.append('default_year', String(defaultYear))
  }
  if (sheetPlan && sheetPlan.length) {
    form.append('sheet_plan', JSON.stringify(sheetPlan))
  }
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

export async function importContestRecords(file: File): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart('/admin/records/contest/import', form)
}

export async function deleteStudent(studentNo: string): Promise<unknown> {
  return requestJson(`/admin/students/${encodeURIComponent(studentNo)}`, { method: 'DELETE' })
}

export async function deleteContestRecord(recordId: string): Promise<unknown> {
  return requestJson(`/admin/records/contest/${recordId}`, { method: 'DELETE' })
}

export async function listDeletedStudents(): Promise<unknown[]> {
  return requestJson('/admin/deleted/students', { method: 'GET' })
}

export async function listDeletedContestRecords(): Promise<unknown[]> {
  return requestJson('/admin/deleted/records/contest', { method: 'GET' })
}

export async function purgeStudent(studentNo: string): Promise<unknown> {
  return requestJson(`/admin/purge/students/${encodeURIComponent(studentNo)}`, { method: 'DELETE' })
}

export async function purgeContestRecord(recordId: string): Promise<unknown> {
  return requestJson(`/admin/purge/records/contest/${recordId}`, { method: 'DELETE' })
}

export async function createUser(payload: {
  username: string
  display_name: string
  role: 'student' | 'teacher' | 'reviewer' | 'admin'
  email?: string
}): Promise<{ user_id?: string; invite_sent: boolean }> {
  return requestJson('/admin/users', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function getPasswordPolicy(): Promise<{
  min_length: number
  require_uppercase: boolean
  require_lowercase: boolean
  require_digit: boolean
  require_symbol: boolean
}> {
  return requestJson('/admin/password-policy', { method: 'GET' })
}

export async function updatePasswordPolicy(payload: {
  min_length: number
  require_uppercase: boolean
  require_lowercase: boolean
  require_digit: boolean
  require_symbol: boolean
}): Promise<{
  min_length: number
  require_uppercase: boolean
  require_lowercase: boolean
  require_digit: boolean
  require_symbol: boolean
}> {
  return requestJson('/admin/password-policy', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export type LaborHourRule = {
  base_hours_a: number
  base_hours_b: number
  national_leader_hours: number
  national_member_hours: number
  provincial_leader_hours: number
  provincial_member_hours: number
  school_leader_hours: number
  school_member_hours: number
}

export async function getLaborHourRules(): Promise<LaborHourRule> {
  return requestJson('/admin/labor-hour-rules', { method: 'GET' })
}

export async function updateLaborHourRules(payload: LaborHourRule): Promise<LaborHourRule> {
  return requestJson('/admin/labor-hour-rules', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export type ImportTemplateField = {
  field_key: string
  label: string
  column_title: string
  required: boolean
  order_index: number
  description?: string | null
}

export type ImportTemplate = {
  template_key: string
  name: string
  fields: ImportTemplateField[]
}

export async function listImportTemplates(): Promise<ImportTemplate[]> {
  return requestJson('/admin/import-templates', { method: 'GET' })
}

export async function updateImportTemplate(
  templateKey: string,
  payload: { name: string; fields: ImportTemplateField[] },
): Promise<ImportTemplate> {
  return requestJson(`/admin/import-templates/${encodeURIComponent(templateKey)}`, {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export type ExportTemplate = {
  template_key: string
  name: string
  layout: Record<string, unknown>
}

export async function getExportTemplate(templateKey: string): Promise<ExportTemplate> {
  return requestJson(`/admin/export-templates/${encodeURIComponent(templateKey)}`, { method: 'GET' })
}

export async function updateExportTemplate(
  templateKey: string,
  payload: { name: string; layout: Record<string, unknown> },
): Promise<ExportTemplate> {
  return requestJson(`/admin/export-templates/${encodeURIComponent(templateKey)}`, {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function resetUserTotp(username: string): Promise<{ status: string }> {
  return requestJson('/admin/users/reset/totp', {
    method: 'POST',
    body: JSON.stringify({ username }),
  })
}

export async function resetUserPasskey(username: string): Promise<{ status: string }> {
  return requestJson('/admin/users/reset/passkey', {
    method: 'POST',
    body: JSON.stringify({ username }),
  })
}

export async function generateResetCode(payload: {
  username: string
  purpose: 'password' | 'totp' | 'passkey'
}): Promise<{ code?: string; expires_in_minutes: number }> {
  return requestJson('/admin/users/reset/code', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}
