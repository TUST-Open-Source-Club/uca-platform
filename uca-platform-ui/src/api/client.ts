const DEFAULT_BASE = import.meta.env.DEV ? 'http://localhost:8443' : 'https://localhost:8443'

const API_BASE = (import.meta.env.VITE_API_BASE as string | undefined) ?? DEFAULT_BASE

export type ApiError = {
  code: string
  message: string
}

async function parseJson<T>(response: Response): Promise<T> {
  const data = (await response.json()) as T | ApiError
  if (!response.ok) {
    const err = data as ApiError
    throw new Error(err.message || '请求失败')
  }
  return data as T
}

export async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
    ...init,
  })
  return parseJson<T>(response)
}

export async function requestMultipart<T>(path: string, form: FormData): Promise<T> {
  const response = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    credentials: 'include',
    body: form,
  })
  return parseJson<T>(response)
}

export async function downloadFile(path: string, body?: unknown): Promise<void> {
  const response = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: body ? JSON.stringify(body) : undefined,
  })
  if (!response.ok) {
    const err = (await response.json()) as ApiError
    throw new Error(err.message || '下载失败')
  }
  const blob = await response.blob()
  const disposition = response.headers.get('content-disposition')
  const filename = disposition?.match(/filename="(.+)"/)?.[1] ?? 'export.bin'
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  link.remove()
  URL.revokeObjectURL(url)
}
