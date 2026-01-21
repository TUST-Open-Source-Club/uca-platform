import { describe, it, expect, vi, beforeEach } from 'vitest'
import router from '../router'
import { createPinia, setActivePinia } from 'pinia'
import { useAuthStore } from '../stores/auth'

const getCurrentUser = vi.fn()

vi.mock('../api/auth', () => ({
  getCurrentUser: (...args: unknown[]) => getCurrentUser(...args),
}))

const resetRouter = async () => {
  await router.replace('/login')
  await router.isReady()
}

beforeEach(async () => {
  const pinia = createPinia()
  setActivePinia(pinia)
  getCurrentUser.mockReset()
  await resetRouter()
})

describe('Router guards', () => {
  it('redirects to login when session is missing', async () => {
    getCurrentUser.mockRejectedValue(new Error('no session'))
    await router.push('/admin')
    await router.isReady()
    expect(router.currentRoute.value.fullPath).toBe('/login')
  })

  it('allows admin to access admin route', async () => {
    const auth = useAuthStore()
    auth.loggedIn = true
    auth.role = 'admin'
    auth.sessionChecked = true
    await router.push('/admin')
    await router.isReady()
    expect(router.currentRoute.value.fullPath).toBe('/admin')
  })

  it('redirects reviewer away from admin route', async () => {
    const auth = useAuthStore()
    auth.loggedIn = true
    auth.role = 'reviewer'
    auth.sessionChecked = true
    await router.push('/admin')
    await router.isReady()
    expect(router.currentRoute.value.fullPath).toBe('/review')
  })

  it('redirects student away from review route', async () => {
    const auth = useAuthStore()
    auth.loggedIn = true
    auth.role = 'student'
    auth.sessionChecked = true
    await router.push('/review')
    await router.isReady()
    expect(router.currentRoute.value.fullPath).toBe('/student')
  })
})
