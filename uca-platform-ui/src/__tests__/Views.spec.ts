import { describe, it, expect, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import StudentDashboard from '../views/StudentDashboard.vue'
import RecordsView from '../views/RecordsView.vue'
import ReviewDashboard from '../views/ReviewDashboard.vue'
import AdminDashboard from '../views/AdminDashboard.vue'
import ExportView from '../views/ExportView.vue'
import DevicesView from '../views/DevicesView.vue'
import TwoFactorView from '../views/TwoFactorView.vue'
import PurgeView from '../views/PurgeView.vue'
import PublicCompetitionsView from '../views/PublicCompetitionsView.vue'

vi.mock('../api/forms', () => ({
  listFormFieldsByType: vi.fn().mockResolvedValue([
    {
      id: '1',
      form_type: 'volunteer',
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
  queryVolunteer: vi.fn().mockResolvedValue([]),
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
}))

vi.mock('../api/exports', () => ({
  exportSummary: vi.fn().mockResolvedValue({}),
  exportStudent: vi.fn().mockResolvedValue({}),
  exportRecordPdf: vi.fn().mockResolvedValue({}),
}))

vi.mock('../api/attachments', () => ({
  uploadSignature: vi.fn().mockResolvedValue({}),
}))

vi.mock('../api/auth', () => ({
  totpVerify: vi.fn().mockResolvedValue({}),
  listDevices: vi.fn().mockResolvedValue([]),
  regenerateRecoveryCodes: vi.fn().mockResolvedValue({ codes: [] }),
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
}

describe('Views', () => {
  it('renders student dashboard with dynamic options', async () => {
    const wrapper = mount(StudentDashboard, { global: { stubs } })
    await flushPromises()
    expect(wrapper.text()).toContain('学生中心')
    expect(wrapper.text()).toContain('全国大学生数学建模竞赛')
  })

  it('renders records view', () => {
    const wrapper = mount(RecordsView, { global: { stubs } })
    expect(wrapper.text()).toContain('我的记录')
  })

  it('renders review dashboard', () => {
    const wrapper = mount(ReviewDashboard, { global: { stubs } })
    expect(wrapper.text()).toContain('审核中心')
  })

  it('renders admin dashboard', () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [{ path: '/', component: { template: '<div />' } }],
    })
    const wrapper = mount(AdminDashboard, { global: { stubs, plugins: [createPinia(), router] } })
    expect(wrapper.text()).toContain('管理台')
    expect(wrapper.text()).toContain('竞赛名称库导入')
  })

  it('renders export view', () => {
    const wrapper = mount(ExportView, { global: { stubs } })
    expect(wrapper.text()).toContain('导出中心')
  })

  it('renders devices view', () => {
    const wrapper = mount(DevicesView, { global: { stubs } })
    expect(wrapper.text()).toContain('设备与恢复码')
  })

  it('renders purge view', () => {
    const wrapper = mount(PurgeView, { global: { stubs } })
    expect(wrapper.text()).toContain('彻底删除')
  })

  it('renders public competitions view', () => {
    const wrapper = mount(PublicCompetitionsView, { global: { stubs } })
    expect(wrapper.text()).toContain('竞赛清单')
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
