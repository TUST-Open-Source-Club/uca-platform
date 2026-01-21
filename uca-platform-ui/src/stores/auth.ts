import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { CurrentUser } from '../api/auth'
import { getCurrentUser } from '../api/auth'

export const useAuthStore = defineStore('auth', () => {
  const loggedIn = ref(false)
  const role = ref<'student' | 'reviewer' | 'teacher' | 'admin'>('student')
  const user = ref<CurrentUser | null>(null)
  const sessionChecked = ref(false)

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

  const homePath = () => {
    if (role.value === 'admin') return '/admin'
    if (role.value === 'reviewer' || role.value === 'teacher') return '/review'
    return '/student'
  }

  return { loggedIn, role, user, sessionChecked, login, logout, refreshSession, ensureSession, homePath }
})
