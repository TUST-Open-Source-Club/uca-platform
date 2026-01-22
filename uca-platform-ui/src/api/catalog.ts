import { requestJson } from './client'

export type CompetitionItem = {
  id: string
  name: string
  year?: number | null
  category?: string | null
}

export async function listCompetitionsPublic(): Promise<CompetitionItem[]> {
  return requestJson('/competitions', { method: 'GET' })
}
