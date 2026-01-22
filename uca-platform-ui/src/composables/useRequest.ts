import { proxyRefs, ref } from 'vue'
import { ElMessage } from 'element-plus'

export function normalizeError(err: unknown): string {
  if (err instanceof Error) {
    return err.message || '请求失败'
  }
  if (typeof err === 'string') {
    return err
  }
  return '请求失败'
}

export function useRequest() {
  const loading = ref(false)
  const error = ref('')

  const run = async <T>(
    action: () => Promise<T>,
    options?: { successMessage?: string; silent?: boolean },
  ): Promise<T> => {
    error.value = ''
    loading.value = true
    try {
      const result = await action()
      if (options?.successMessage) {
        ElMessage.success(options.successMessage)
      }
      return result
    } catch (err) {
      error.value = normalizeError(err)
      if (!options?.silent) {
        ElMessage.error(error.value)
      }
      throw err
    } finally {
      loading.value = false
    }
  }

  return proxyRefs({ loading, error, run })
}
