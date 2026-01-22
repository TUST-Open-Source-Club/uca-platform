<script setup lang="ts">
import { computed } from 'vue'
import { useAuthStore } from './stores/auth'

const auth = useAuthStore()

const navItems = computed(() => {
  if (!auth.loggedIn) {
    return [
      { path: '/login', label: '登录' },
      { path: '/competitions', label: '竞赛清单' },
    ]
  }

  const items: { path: string; label: string }[] = []

  if (auth.role === 'student') {
    items.push({ path: '/student', label: '学生中心' })
    items.push({ path: '/records', label: '我的记录' })
  }

  if (auth.role === 'teacher' || auth.role === 'reviewer' || auth.role === 'admin') {
    items.push({ path: '/review', label: '审核中心' })
    items.push({ path: '/exports', label: '导出中心' })
  }

  if (auth.role === 'admin') {
    items.push({ path: '/admin', label: '管理台' })
    items.push({ path: '/admin/imports', label: '数据导入' })
    items.push({ path: '/admin/competitions', label: '竞赛库管理' })
    items.push({ path: '/admin/form-fields', label: '模板配置' })
    items.push({ path: '/admin/students', label: '学生名单管理' })
    items.push({ path: '/admin/soft-delete', label: '数据删除（软删除）' })
    items.push({ path: '/admin/users', label: '创建用户 / 邀请' })
    items.push({ path: '/admin/password-policy', label: '密码策略' })
    items.push({ path: '/admin/auth-reset', label: '认证重置' })
    items.push({ path: '/admin/reset-code', label: '一次性重置码' })
  }

  items.push({ path: '/devices', label: '设备与认证' })
  return items
})
</script>

<template>
  <el-container class="app-shell">
    <el-aside class="sidebar" width="260px">
      <div class="brand">Labor Hours Platform</div>
      <el-menu router class="nav" background-color="transparent" text-color="#1f4d63">
        <el-menu-item v-for="item in navItems" :key="item.path" :index="item.path">
          {{ item.label }}
        </el-menu-item>
      </el-menu>
    </el-aside>
    <el-main class="main">
      <RouterView />
    </el-main>
  </el-container>
</template>
