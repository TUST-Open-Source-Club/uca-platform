import { requestJson } from './client'

export async function createContest(payload: {
  contest_name: string
  contest_year?: number | null
  contest_category?: string | null
  contest_level?: string | null
  contest_role?: string | null
  award_level: string
  award_date?: string | null
  self_hours: number
  custom_fields?: Record<string, string>
}): Promise<unknown> {
  return requestJson('/records/contest', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function queryContest(status?: string): Promise<unknown[]> {
  return requestJson('/records/contest/query', {
    method: 'POST',
    body: JSON.stringify({ status }),
  })
}

export async function reviewContest(recordId: string, payload: unknown): Promise<unknown> {
  return requestJson(`/records/contest/${recordId}/review`, {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}
