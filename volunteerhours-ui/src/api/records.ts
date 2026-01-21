import { requestJson } from './client'

export async function createVolunteer(payload: {
  title: string
  description: string
  self_hours: number
}): Promise<unknown> {
  return requestJson('/records/volunteer', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function createContest(payload: {
  contest_name: string
  award_level: string
  self_hours: number
}): Promise<unknown> {
  return requestJson('/records/contest', {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function queryVolunteer(status?: string): Promise<unknown[]> {
  return requestJson('/records/volunteer/query', {
    method: 'POST',
    body: JSON.stringify({ status }),
  })
}

export async function queryContest(status?: string): Promise<unknown[]> {
  return requestJson('/records/contest/query', {
    method: 'POST',
    body: JSON.stringify({ status }),
  })
}

export async function reviewVolunteer(recordId: string, payload: unknown): Promise<unknown> {
  return requestJson(`/records/volunteer/${recordId}/review`, {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}

export async function reviewContest(recordId: string, payload: unknown): Promise<unknown> {
  return requestJson(`/records/contest/${recordId}/review`, {
    method: 'POST',
    body: JSON.stringify(payload),
  })
}
