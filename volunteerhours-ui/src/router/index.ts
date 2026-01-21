import { createRouter, createWebHistory } from 'vue-router'

import AdminDashboard from '../views/AdminDashboard.vue'
import DevicesView from '../views/DevicesView.vue'
import ExportView from '../views/ExportView.vue'
import LoginView from '../views/LoginView.vue'
import RecordsView from '../views/RecordsView.vue'
import ReviewDashboard from '../views/ReviewDashboard.vue'
import StudentDashboard from '../views/StudentDashboard.vue'
import TwoFactorView from '../views/TwoFactorView.vue'

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

export default router
