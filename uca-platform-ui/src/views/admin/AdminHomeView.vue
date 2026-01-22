<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../../stores/auth'

const router = useRouter()
const auth = useAuthStore()

onMounted(() => {
  void auth.ensureConfig()
})

const sections = computed(() => {
  const items = [
    { title: '数据导入', desc: '学生名单、竞赛库与记录导入。', path: '/admin/imports' },
    { title: '竞赛库管理', desc: '新增/查看竞赛名称库。', path: '/admin/competitions' },
    { title: '模板配置', desc: '维护竞赛表单字段、导出模板与学时规则。', path: '/admin/form-fields' },
    { title: '学生名单管理', desc: '表格化维护学生名单并批量删除。', path: '/admin/students' },
    { title: '清理已删除', desc: '删除未审核记录并进入彻底删除。', path: '/purge' },
    { title: '创建用户', desc: '新增用户并配置认证方式。', path: '/admin/users' },
    { title: '密码策略', desc: '设置学生密码校验规则。', path: '/admin/password-policy' },
  ]
  if (auth.resetDelivery === 'code') {
    items.push({ title: '一次性重置码', desc: '生成学生密码/认证重置码。', path: '/admin/reset-code' })
  } else {
    items.push({ title: '认证重置', desc: '发送非学生 TOTP/Passkey 重置链接。', path: '/admin/auth-reset' })
  }
  return items
})

const go = (path: string) => {
  void router.push(path)
}
</script>

<template>
  <section class="hero">
    <h1>管理台</h1>
    <p>按功能进入具体管理页面。</p>
  </section>

  <div class="card-grid">
    <el-card v-for="section in sections" :key="section.path" class="card">
      <h3>{{ section.title }}</h3>
      <p>{{ section.desc }}</p>
      <el-button type="primary" @click="go(section.path)">进入</el-button>
    </el-card>
  </div>
</template>
