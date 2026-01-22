import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { CurrentUser } from '../api/auth'
import { bootstrapStatus, getCurrentUser } from '../api/auth'

export const useAuthStore = defineStore('auth', () => {
  const loggedIn = ref(false)
  const role = ref<'student' | 'reviewer' | 'teacher' | 'admin'>('student')
  const user = ref<CurrentUser | null>(null)
  const sessionChecked = ref(false)
  const bootstrapChecked = ref(false)
  const bootstrapReady = ref<boolean | null>(null)
  const bootstrapNeedsTotp = ref<boolean | null>(null)

  const login = (nextRole?: typeof role.value) => {
    loggedIn.value = true
    if (nextRole) {
      role.value = nextRole
    }
  }

  const logout = () => {
    loggedIn.value = false
    role.value = 'student'
    user.value = null
    sessionChecked.value = false
  }

  const refreshSession = async () => {
    try {
      const profile = await getCurrentUser()
      user.value = profile
      loggedIn.value = true
      role.value = profile.role
      sessionChecked.value = true
      return profile
    } catch {
      loggedIn.value = false
      role.value = 'student'
      user.value = null
      sessionChecked.value = true
      return null
    }
  }

  const ensureSession = async () => {
    if (sessionChecked.value) {
      return user.value
    }
    return refreshSession()
  }

  const refreshBootstrap = async () => {
    try {
      const status = await bootstrapStatus()
      bootstrapReady.value = status.ready
      bootstrapNeedsTotp.value = status.needs_totp
      bootstrapChecked.value = true
      return status
    } catch {
      bootstrapReady.value = null
      bootstrapNeedsTotp.value = null
      bootstrapChecked.value = true
      return null
    }
  }

  const ensureBootstrap = async () => {
    if (bootstrapChecked.value) {
      return bootstrapReady.value === null
        ? null
        : { ready: bootstrapReady.value, needs_totp: bootstrapNeedsTotp.value ?? false }
    }
    return refreshBootstrap()
  }

  const homePath = () => {
    if (role.value === 'admin') return '/admin'
    if (role.value === 'reviewer' || role.value === 'teacher') return '/review'
    return '/student'
  }

  return {
    loggedIn,
    role,
    user,
    sessionChecked,
    bootstrapChecked,
    bootstrapReady,
    bootstrapNeedsTotp,
    login,
    logout,
    refreshSession,
    ensureSession,
    refreshBootstrap,
    ensureBootstrap,
    homePath,
  }
})
