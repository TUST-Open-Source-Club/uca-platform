import { requestJson } from './client'

export type CompetitionItem = {
  id: string
  name: string
}

export async function listCompetitionsPublic(): Promise<CompetitionItem[]> {
  return requestJson('/competitions', { method: 'GET' })
}
