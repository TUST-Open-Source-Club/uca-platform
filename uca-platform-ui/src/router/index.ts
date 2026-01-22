import { createRouter, createWebHistory } from 'vue-router'

import AdminDashboard from '../views/AdminDashboard.vue'
import DevicesView from '../views/DevicesView.vue'
import ExportView from '../views/ExportView.vue'
import LoginView from '../views/LoginView.vue'
import InviteView from '../views/InviteView.vue'
import ResetView from '../views/ResetView.vue'
import PasswordResetRequestView from '../views/PasswordResetRequestView.vue'
import PasswordResetView from '../views/PasswordResetView.vue'
import RecordsView from '../views/RecordsView.vue'
import ReviewDashboard from '../views/ReviewDashboard.vue'
import StudentDashboard from '../views/StudentDashboard.vue'
import TwoFactorView from '../views/TwoFactorView.vue'
import PurgeView from '../views/PurgeView.vue'
import PublicCompetitionsView from '../views/PublicCompetitionsView.vue'
import SetupView from '../views/SetupView.vue'
import { useAuthStore } from '../stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', redirect: '/login' },
    { path: '/login', component: LoginView },
    { path: '/setup', component: SetupView },
    { path: '/invite', component: InviteView },
    { path: '/reset', component: ResetView },
    { path: '/password-reset/request', component: PasswordResetRequestView },
    { path: '/password-reset', component: PasswordResetView },
    { path: '/2fa', component: TwoFactorView },
    { path: '/competitions', component: PublicCompetitionsView },
    { path: '/student', component: StudentDashboard },
    { path: '/records', component: RecordsView },
    { path: '/review', component: ReviewDashboard },
    { path: '/admin', component: AdminDashboard },
    { path: '/purge', component: PurgeView },
    { path: '/exports', component: ExportView },
    { path: '/devices', component: DevicesView },
  ],
})

router.beforeEach(async (to) => {
  const auth = useAuthStore()
  const publicRoutes = [
    '/login',
    '/2fa',
    '/competitions',
    '/setup',
    '/invite',
    '/reset',
    '/password-reset',
    '/password-reset/request',
  ]
  if (publicRoutes.includes(to.path)) {
    const status = await auth.ensureBootstrap()
    if (status && status.needs_totp && to.path !== '/setup') return '/setup'
    if (status && status.ready === false && to.path !== '/setup') return '/setup'
    if (status && status.ready === true && to.path === '/setup') return '/login'
    return true
  }
  await auth.ensureSession()
  if (!auth.loggedIn) return '/login'
  if (to.path === '/admin' && auth.role !== 'admin') return auth.homePath()
  if (to.path === '/purge' && auth.role !== 'admin') return auth.homePath()
  if (to.path === '/review' && auth.role === 'student') return auth.homePath()
  return true
})

export default router
