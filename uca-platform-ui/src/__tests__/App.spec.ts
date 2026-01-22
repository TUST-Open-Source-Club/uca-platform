import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from '../App.vue'

describe('App', () => {
  it('renders brand', () => {
    setActivePinia(createPinia())
    const wrapper = mount(App, {
      global: {
        plugins: [createPinia()],
        stubs: {
          RouterLink: { template: '<a><slot /></a>' },
          RouterView: { template: '<div />' },
          'el-container': { template: '<div><slot /></div>' },
          'el-main': { template: '<main><slot /></main>' },
          'el-aside': { template: '<aside><slot /></aside>' },
          'el-menu': { template: '<nav><slot /></nav>' },
          'el-menu-item': { template: '<div><slot /></div>' },
        },
      },
    })
    expect(wrapper.text()).toContain('Labor Hours Platform')
  })
})
