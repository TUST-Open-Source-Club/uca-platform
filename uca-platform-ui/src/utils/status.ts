export const statusLabels: Record<string, string> = {
  submitted: '已提交',
  first_reviewed: '已初审',
  final_reviewed: '已复审',
  rejected: '不通过',
}

export const matchLabels: Record<string, string> = {
  matched: '已匹配',
  unmatched: '未匹配',
}

export const formatStatus = (value?: string | null) => statusLabels[value ?? ''] ?? (value || '-')
export const formatMatchStatus = (value?: string | null) => matchLabels[value ?? ''] ?? (value || '-')
