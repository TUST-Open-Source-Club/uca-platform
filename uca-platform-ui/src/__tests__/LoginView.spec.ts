import { describe, it, expect, vi } from 'vitest'
import { defineComponent } from 'vue'
import { mount, flushPromises } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia, setActivePinia } from 'pinia'
import LoginView from '../views/LoginView.vue'

vi.mock('../api/catalog', () => ({
  listCompetitionsPublic: vi.fn().mockResolvedValue([{ id: 'c1', name: '全国大学生数学建模竞赛' }]),
}))

describe('LoginView', () => {
  it('shows login options', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const router = createRouter({
      history: createWebHistory(),
      routes: [
        { path: '/', component: { template: '<div />' } },
        { path: '/password-reset/request', component: { template: '<div />' } },
        { path: '/reset-code', component: { template: '<div />' } },
      ],
    })
    const ElTableStub = defineComponent({
      props: ['data'],
      template: '<div><div v-for="row in data">{{ row.name }}</div><slot /></div>',
    })

    const wrapper = mount(LoginView, {
      global: {
        plugins: [pinia, router],
        stubs: {
          'el-button': { template: '<button><slot /></button>' },
          'el-card': { template: '<div><slot /></div>' },
          'el-input': { template: '<input />' },
          'el-form': { template: '<form><slot /></form>' },
          'el-form-item': { template: '<div><slot /></div>' },
          'el-select': { template: '<select><slot /></select>' },
          'el-option': { template: '<option><slot /></option>' },
          'el-alert': { template: '<div />' },
          'el-table': ElTableStub,
          'el-table-column': { template: '<col />' },
          'el-empty': { template: '<div />' },
        },
      },
    })
    await flushPromises()
    expect(wrapper.text()).toContain('Passkey 登录')
    expect(wrapper.text()).toContain('TOTP 登录')
    expect(wrapper.text()).toContain('全国大学生数学建模竞赛')
  })
})
