import { requestJson } from './client'

export type FormField = {
  id: string
  form_type: string
  field_key: string
  label: string
  field_type: string
  required: boolean
  order_index: number
}

export async function listFormFieldsByType(formType: string): Promise<FormField[]> {
  return requestJson(`/forms/${formType}/fields`, { method: 'GET' })
}
