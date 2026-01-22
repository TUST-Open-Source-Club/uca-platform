import { createRouter, createWebHistory } from 'vue-router'

import AdminDashboard from '../views/AdminDashboard.vue'
import AdminHomeView from '../views/admin/AdminHomeView.vue'
import AdminImportsView from '../views/admin/AdminImportsView.vue'
import AdminCompetitionsView from '../views/admin/AdminCompetitionsView.vue'
import AdminFormFieldsView from '../views/admin/AdminFormFieldsView.vue'
import AdminSoftDeleteView from '../views/admin/AdminSoftDeleteView.vue'
import AdminUsersView from '../views/admin/AdminUsersView.vue'
import AdminStudentsView from '../views/admin/AdminStudentsView.vue'
import AdminPasswordPolicyView from '../views/admin/AdminPasswordPolicyView.vue'
import AdminAuthResetView from '../views/admin/AdminAuthResetView.vue'
import AdminResetCodeView from '../views/admin/AdminResetCodeView.vue'
import DevicesView from '../views/DevicesView.vue'
import ExportView from '../views/ExportView.vue'
import LoginView from '../views/LoginView.vue'
import InviteView from '../views/InviteView.vue'
import ResetView from '../views/ResetView.vue'
import ResetCodeView from '../views/ResetCodeView.vue'
import PasswordResetRequestView from '../views/PasswordResetRequestView.vue'
import PasswordResetView from '../views/PasswordResetView.vue'
import RecordsView from '../views/RecordsView.vue'
import ReviewDashboard from '../views/ReviewDashboard.vue'
import StudentDashboard from '../views/StudentDashboard.vue'
import TwoFactorView from '../views/TwoFactorView.vue'
import PurgeView from '../views/PurgeView.vue'
import PublicCompetitionsView from '../views/PublicCompetitionsView.vue'
import SetupView from '../views/SetupView.vue'
import ProfileView from '../views/ProfileView.vue'
import { useAuthStore } from '../stores/auth'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', redirect: '/login' },
    { path: '/login', component: LoginView },
    { path: '/setup', component: SetupView },
    { path: '/invite', component: InviteView },
    { path: '/reset', component: ResetView },
    { path: '/reset-code', component: ResetCodeView },
    { path: '/password-reset/request', component: PasswordResetRequestView },
    { path: '/password-reset', component: PasswordResetView },
    { path: '/2fa', component: TwoFactorView },
    { path: '/competitions', component: PublicCompetitionsView },
    { path: '/student', component: StudentDashboard },
    { path: '/records', component: RecordsView },
    { path: '/review', component: ReviewDashboard },
    { path: '/profile', component: ProfileView },
    { path: '/admin', component: AdminHomeView },
    { path: '/admin/home', component: AdminDashboard },
    { path: '/admin/imports', component: AdminImportsView },
    { path: '/admin/competitions', component: AdminCompetitionsView },
    { path: '/admin/form-fields', component: AdminFormFieldsView },
    { path: '/admin/soft-delete', component: AdminSoftDeleteView },
    { path: '/admin/users', component: AdminUsersView },
    { path: '/admin/students', component: AdminStudentsView },
    { path: '/admin/password-policy', component: AdminPasswordPolicyView },
    { path: '/admin/auth-reset', component: AdminAuthResetView },
    { path: '/admin/reset-code', component: AdminResetCodeView },
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
    '/reset-code',
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
  if (auth.role === 'student' && auth.mustChangePassword && to.path !== '/student') {
    return '/student'
  }
  if ((to.path === '/purge' || to.path.startsWith('/admin')) && auth.role !== 'admin') {
    return auth.homePath()
  }
  if (to.path === '/review' && auth.role === 'student') return auth.homePath()
  return true
})

export default router
