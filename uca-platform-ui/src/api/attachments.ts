import { requestMultipart } from './client'

export async function uploadContestAttachment(recordId: string, file: File): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart(`/attachments/contest/${recordId}`, form)
}

export async function uploadSignature(
  recordType: string,
  recordId: string,
  stage: string,
  file: File,
): Promise<unknown> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart(`/signatures/${recordType}/${recordId}/${stage}`, form)
}
