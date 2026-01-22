import { downloadFile } from './client'

export async function exportSummary(filters: Record<string, unknown>): Promise<void> {
  return downloadFile('/export/summary/excel', filters)
}

export async function exportStudent(studentNo: string): Promise<void> {
  return downloadFile(`/export/student/${studentNo}/excel`)
}

export async function exportRecordPdf(recordType: string, recordId: string): Promise<void> {
  return downloadFile(`/export/record/${recordType}/${recordId}/pdf`)
}

export async function exportLaborHoursPdf(studentNo: string): Promise<void> {
  return downloadFile(`/export/labor-hours/${encodeURIComponent(studentNo)}/pdf`)
}
