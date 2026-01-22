import { describe, it, expect, vi, beforeEach } from 'vitest'
import { defineComponent, h } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { createMemoryHistory, createRouter } from 'vue-router'
import { createPinia, setActivePinia } from 'pinia'
import LoginView from '../views/LoginView.vue'
import TwoFactorView from '../views/TwoFactorView.vue'

vi.mock('element-plus', () => ({
  ElMessage: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

const passkeyStart = vi.fn()
const passkeyFinish = vi.fn()
const totpVerify = vi.fn()
const passwordLogin = vi.fn()
const loginOptions = vi.fn()
const getCurrentUser = vi.fn()

vi.mock('../api/auth', () => ({
  passkeyStart: (...args: unknown[]) => passkeyStart(...args),
  passkeyFinish: (...args: unknown[]) => passkeyFinish(...args),
  totpVerify: (...args: unknown[]) => totpVerify(...args),
  passwordLogin: (...args: unknown[]) => passwordLogin(...args),
  loginOptions: (...args: unknown[]) => loginOptions(...args),
  getCurrentUser: (...args: unknown[]) => getCurrentUser(...args),
  bootstrapStatus: vi.fn().mockResolvedValue({ ready: true, needs_totp: false }),
}))

vi.mock('../api/catalog', () => ({
  listCompetitionsPublic: vi.fn().mockResolvedValue([]),
}))

vi.mock('../utils/webauthn', () => ({
  normalizeRequestOptions: (options: unknown) => options,
  credentialToJson: (credential: unknown) => credential,
}))

const ElFormStub = defineComponent({
  setup(_props, { slots, expose }) {
    const validate = async (cb: (valid: boolean) => void) => cb(true)
    expose({ validate })
    return () => h('form', slots.default?.())
  },
})

const stubs = {
  'el-card': { template: '<div><slot /></div>' },
  'el-form': ElFormStub,
  'el-form-item': { template: '<div><slot /></div>' },
  'el-input': { template: '<input />' },
  'el-input-number': { template: '<input />' },
  'el-button': { template: '<button><slot /></button>' },
  'el-select': { template: '<select><slot /></select>' },
  'el-option': { template: '<option><slot /></option>' },
  'el-alert': { template: '<div />' },
  'el-table': { template: '<table><slot /></table>' },
  'el-table-column': { template: '<col />' },
  'el-empty': { template: '<div />' },
}

const buildRouter = () =>
  createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', component: { template: '<div />' } },
      { path: '/student', component: { template: '<div />' } },
      { path: '/review', component: { template: '<div />' } },
      { path: '/admin', component: { template: '<div />' } },
      { path: '/password-reset/request', component: { template: '<div />' } },
      { path: '/reset-code', component: { template: '<div />' } },
    ],
  })

beforeEach(() => {
  passkeyStart.mockReset()
  passkeyFinish.mockReset()
  totpVerify.mockReset()
  passwordLogin.mockReset()
  loginOptions.mockResolvedValue({ methods: ['passkey', 'totp', 'password'] })
  getCurrentUser.mockReset()
  const pinia = createPinia()
  setActivePinia(pinia)
  Object.defineProperty(globalThis, 'PublicKeyCredential', {
    value: class {},
    configurable: true,
  })
  Object.defineProperty(navigator, 'credentials', {
    value: { get: vi.fn().mockResolvedValue({ id: 'cred' }) },
    configurable: true,
  })
})

describe('Auth flow', () => {
  it('logs in with totp and redirects to student home', async () => {
    totpVerify.mockResolvedValue({ ok: true })
    getCurrentUser.mockResolvedValue({ id: 'u1', username: 'u1', display_name: 'u1', role: 'student' })
    const router = buildRouter()
    const pinia = createPinia()
    setActivePinia(pinia)
    await router.push('/')
    await router.isReady()
    const wrapper = mount(LoginView, { global: { stubs, plugins: [pinia, router] } })
    ;(wrapper.vm as any).form.username = 'u1'
    ;(wrapper.vm as any).form.code = '123456'
    ;(wrapper.vm as any).form.method = 'totp'
    const submit = wrapper.findAll('button').find((btn) => btn.text() === '进入认证')
    await submit?.trigger('click')
    await flushPromises()
    expect(totpVerify).toHaveBeenCalledWith('u1', '123456')
    expect(router.currentRoute.value.fullPath).toBe('/student')
  })

  it('logs in with passkey and redirects to admin home', async () => {
    passkeyStart.mockResolvedValue({ session_id: 's1', public_key: {} })
    passkeyFinish.mockResolvedValue({ ok: true })
    getCurrentUser.mockResolvedValue({ id: 'u3', username: 'u3', display_name: 'u3', role: 'admin' })
    const router = buildRouter()
    const pinia = createPinia()
    setActivePinia(pinia)
    await router.push('/')
    await router.isReady()
    const wrapper = mount(LoginView, { global: { stubs, plugins: [pinia, router] } })
    ;(wrapper.vm as any).form.username = 'u3'
    ;(wrapper.vm as any).form.method = 'passkey'
    const submit = wrapper.findAll('button').find((btn) => btn.text() === '进入认证')
    await submit?.trigger('click')
    await flushPromises()
    expect(passkeyStart).toHaveBeenCalledWith('u3')
    expect(passkeyFinish).toHaveBeenCalled()
    expect(router.currentRoute.value.fullPath).toBe('/admin')
  })

  it('verifies totp in two factor view and redirects', async () => {
    totpVerify.mockResolvedValue({ ok: true })
    getCurrentUser.mockResolvedValue({ id: 'u4', username: 'u4', display_name: 'u4', role: 'student' })
    const router = buildRouter()
    const pinia = createPinia()
    setActivePinia(pinia)
    await router.push('/')
    await router.isReady()
    const wrapper = mount(TwoFactorView, { global: { stubs, plugins: [pinia, router] } })
    ;(wrapper.vm as any).form.username = 'u4'
    ;(wrapper.vm as any).form.code = '123456'
    await wrapper.find('button').trigger('click')
    await flushPromises()
    expect(totpVerify).toHaveBeenCalledWith('u4', '123456')
    expect(router.currentRoute.value.fullPath).toBe('/student')
  })
})
