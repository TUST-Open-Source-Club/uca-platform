import { requestJson, requestMultipart } from './client'

export type SignatureProfile = {
  uploaded: boolean
  signature_path?: string | null
}

export async function getSignatureProfile(): Promise<SignatureProfile> {
  return requestJson('/profile/signature', { method: 'GET' })
}

export async function uploadSignatureImage(file: File): Promise<SignatureProfile> {
  const form = new FormData()
  form.append('file', file)
  return requestMultipart('/profile/signature', form)
}
