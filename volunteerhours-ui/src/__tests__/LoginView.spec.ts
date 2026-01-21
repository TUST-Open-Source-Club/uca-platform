import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createRouter, createWebHistory } from 'vue-router'
import { createPinia, setActivePinia } from 'pinia'
import LoginView from '../views/LoginView.vue'

describe('LoginView', () => {
  it('shows login options', () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const router = createRouter({
      history: createWebHistory(),
      routes: [{ path: '/', component: { template: '<div />' } }],
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
        },
      },
    })
    expect(wrapper.text()).toContain('Passkey 登录')
    expect(wrapper.text()).toContain('TOTP 登录')
  })
})
