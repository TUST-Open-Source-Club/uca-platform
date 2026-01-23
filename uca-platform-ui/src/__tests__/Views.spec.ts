import { describe, it, expect, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import StudentDashboard from '../views/StudentDashboard.vue'
import RecordsView from '../views/RecordsView.vue'
import ReviewDashboard from '../views/ReviewDashboard.vue'
import AdminHomeView from '../views/admin/AdminHomeView.vue'
import ExportView from '../views/ExportView.vue'
import DevicesView from '../views/DevicesView.vue'
import TwoFactorView from '../views/TwoFactorView.vue'
import PurgeView from '../views/PurgeView.vue'
import PublicCompetitionsView from '../views/PublicCompetitionsView.vue'
import SetupView from '../views/SetupView.vue'

vi.mock('qrcode', () => ({
  default: {
    toDataURL: vi.fn().mockResolvedValue('data:image/png;base64,stub'),
  },
}))

vi.mock('../api/forms', () => ({
  listFormFieldsByType: vi.fn().mockResolvedValue([
    {
      id: '1',
      form_type: 'contest',
      field_key: 'location',
      label: '地点',
      field_type: 'text',
      required: true,
      order_index: 1,
    },
  ]),
}))

vi.mock('../api/catalog', () => ({
  listCompetitionsPublic: vi.fn().mockResolvedValue([{ id: 'c1', name: '全国大学生数学建模竞赛' }]),
}))

vi.mock('../api/records', () => ({
  createVolunteer: vi.fn().mockResolvedValue({}),
  createContest: vi.fn().mockResolvedValue({}),
  queryContest: vi.fn().mockResolvedValue([]),
  reviewVolunteer: vi.fn().mockResolvedValue({}),
  reviewContest: vi.fn().mockResolvedValue({}),
}))

vi.mock('../api/admin', () => ({
  createCompetition: vi.fn().mockResolvedValue({}),
  createFormField: vi.fn().mockResolvedValue({}),
  importCompetitions: vi.fn().mockResolvedValue({}),
  importVolunteerRecords: vi.fn().mockResolvedValue({}),
  importContestRecords: vi.fn().mockResolvedValue({}),
  listCompetitions: vi.fn().mockResolvedValue([]),
  listFormFields: vi.fn().mockResolvedValue([]),
  deleteStudent: vi.fn().mockResolvedValue({}),
  deleteVolunteerRecord: vi.fn().mockResolvedValue({}),
  deleteContestRecord: vi.fn().mockResolvedValue({}),
  listDeletedStudents: vi.fn().mockResolvedValue([]),
  listDeletedVolunteerRecords: vi.fn().mockResolvedValue([]),
  listDeletedContestRecords: vi.fn().mockResolvedValue([]),
  purgeStudent: vi.fn().mockResolvedValue({}),
  purgeVolunteerRecord: vi.fn().mockResolvedValue({}),
  purgeContestRecord: vi.fn().mockResolvedValue({}),
  createUser: vi.fn().mockResolvedValue({ invite_sent: true }),
  getPasswordPolicy: vi.fn().mockResolvedValue({
    min_length: 8,
    require_uppercase: false,
    require_lowercase: false,
    require_digit: true,
    require_symbol: false,
  }),
  updatePasswordPolicy: vi.fn().mockResolvedValue({
    min_length: 8,
    require_uppercase: false,
    require_lowercase: false,
    require_digit: true,
    require_symbol: false,
  }),
  resetUserTotp: vi.fn().mockResolvedValue({ status: 'ok' }),
  resetUserPasskey: vi.fn().mockResolvedValue({ status: 'ok' }),
  generateResetCode: vi.fn().mockResolvedValue({ code: 'RESET123', expires_in_minutes: 1440 }),
}))

vi.mock('../api/exports', () => ({
  exportSummary: vi.fn().mockResolvedValue({}),
  exportStudent: vi.fn().mockResolvedValue({}),
  exportRecordPdf: vi.fn().mockResolvedValue({}),
  exportLaborHoursPdf: vi.fn().mockResolvedValue({}),
  exportLaborHoursSummaryExcel: vi.fn().mockResolvedValue({}),
}))

vi.mock('../api/attachments', () => ({
  uploadSignature: vi.fn().mockResolvedValue({}),
}))

vi.mock('../api/students', () => ({
  getCurrentStudent: vi.fn().mockResolvedValue({
    student_no: '2023001',
    name: '张三',
    gender: '男',
    department: '信息学院',
    major: '软件工程',
    class_name: '软工1班',
    phone: '13800000000',
    allow_password_login: true,
  }),
}))

vi.mock('../api/auth', () => ({
  totpVerify: vi.fn().mockResolvedValue({}),
  listDevices: vi.fn().mockResolvedValue([]),
  getCurrentUser: vi.fn().mockResolvedValue({ id: 'u1', username: 'u1', display_name: 'u1', role: 'reviewer' }),
  bootstrapStatus: vi.fn().mockResolvedValue({ ready: true, needs_totp: false }),
  totpEnrollStart: vi.fn().mockResolvedValue({ enrollment_id: 'e1', otpauth_url: 'otpauth://totp/demo' }),
  totpEnrollFinish: vi.fn().mockResolvedValue({ status: 'ok' }),
}))

const stubs = {
  'el-card': { template: '<div><slot /></div>' },
  'el-form': { template: '<form><slot /></form>' },
  'el-form-item': { template: '<div><slot /></div>' },
  'el-input': { template: '<input />' },
  'el-input-number': { template: '<input />' },
  'el-button': { template: '<button><slot /></button>' },
  'el-select': { template: '<select><slot /></select>' },
  'el-option': { props: ['label'], template: '<option>{{ label }}</option>' },
  'el-alert': { template: '<div />' },
  'el-upload': { template: '<div><slot /></div>' },
  'el-table': { template: '<table><slot /></table>' },
  'el-table-column': { template: '<col />' },
  'el-empty': { template: '<div />' },
  'el-divider': { template: '<hr />' },
  'el-dialog': { template: '<div><slot /><slot name=\"footer\" /></div>' },
  'el-drawer': { template: '<div><slot /></div>' },
  'el-descriptions': { template: '<div><slot /></div>' },
  'el-descriptions-item': { template: '<div><slot /></div>' },
  'el-pagination': { template: '<div />' },
  'el-date-picker': { template: '<input />' },
  'el-image': { template: '<img />' },
  'el-link': { template: '<a><slot /></a>' },
}

describe('Views', () => {
  it('renders student dashboard with dynamic options', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const wrapper = mount(StudentDashboard, { global: { stubs, plugins: [pinia] } })
    await flushPromises()
    expect(wrapper.text()).toContain('学生中心')
    expect(wrapper.text()).toContain('全国大学生数学建模竞赛')
  })

  it('renders records view', () => {
    const wrapper = mount(RecordsView, { global: { stubs } })
    expect(wrapper.text()).toContain('我的记录')
  })

  it('renders review dashboard', () => {
    const wrapper = mount(ReviewDashboard, { global: { stubs, plugins: [createPinia()] } })
    expect(wrapper.text()).toContain('审核中心')
  })

  it('renders admin home view', () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/', component: { template: '<div />' } }],
    })
    const wrapper = mount(AdminHomeView, { global: { stubs, plugins: [createPinia(), router] } })
    expect(wrapper.text()).toContain('管理台')
    expect(wrapper.text()).toContain('数据导入')
  })

  it('renders export view', () => {
    const wrapper = mount(ExportView, { global: { stubs } })
    expect(wrapper.text()).toContain('导出中心')
  })

  it('renders devices view', () => {
    const wrapper = mount(DevicesView, { global: { stubs, plugins: [createPinia()] } })
    expect(wrapper.text()).toContain('设备与认证')
  })

  it('renders purge view', () => {
    const wrapper = mount(PurgeView, { global: { stubs } })
    expect(wrapper.text()).toContain('彻底删除')
  })

  it('renders public competitions view', () => {
    const wrapper = mount(PublicCompetitionsView, { global: { stubs } })
    expect(wrapper.text()).toContain('竞赛清单')
  })

  it('renders setup view', () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [
        { path: '/', component: { template: '<div />' } },
        { path: '/login', component: { template: '<div />' } },
      ],
    })
    const wrapper = mount(SetupView, { global: { stubs, plugins: [createPinia(), router] } })
    expect(wrapper.text()).toContain('设置向导')
  })

  it('renders two factor view', async () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/', component: { template: '<div />' } }],
    })
    await router.push('/')
    await router.isReady()
    const wrapper = mount(TwoFactorView, { global: { stubs, plugins: [createPinia(), router] } })
    expect(wrapper.text()).toContain('二次验证')
  })
})
