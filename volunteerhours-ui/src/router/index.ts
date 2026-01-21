import { createRouter, createWebHistory } from 'vue-router'

import AdminDashboard from '../views/AdminDashboard.vue'
import DevicesView from '../views/DevicesView.vue'
import ExportView from '../views/ExportView.vue'
import LoginView from '../views/LoginView.vue'
import RecordsView from '../views/RecordsView.vue'
import ReviewDashboard from '../views/ReviewDashboard.vue'
import StudentDashboard from '../views/StudentDashboard.vue'
import TwoFactorView from '../views/TwoFactorView.vue'
import { useAuthStore } from '../stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', redirect: '/login' },
    { path: '/login', component: LoginView },
    { path: '/2fa', component: TwoFactorView },
    { path: '/student', component: StudentDashboard },
    { path: '/records', component: RecordsView },
    { path: '/review', component: ReviewDashboard },
    { path: '/admin', component: AdminDashboard },
    { path: '/exports', component: ExportView },
    { path: '/devices', component: DevicesView },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  const publicRoutes = ['/login', '/2fa']
  if (publicRoutes.includes(to.path)) return true
  await auth.ensureSession()
  if (!auth.loggedIn) return '/login'
  if (to.path === '/admin' && auth.role !== 'admin') return auth.homePath()
  if (to.path === '/review' && auth.role === 'student') return auth.homePath()
  return true
})

export default router
