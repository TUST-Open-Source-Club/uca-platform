<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
const sections = [
  { title: '数据导入', desc: '学生名单、竞赛库与记录导入。', path: '/admin/imports' },
  { title: '竞赛库管理', desc: '新增/查看竞赛名称库。', path: '/admin/competitions' },
  { title: '模板配置', desc: '维护竞赛表单字段与导出模板、学时规则。', path: '/admin/form-fields' },
  { title: '学生名单管理', desc: '表格化维护学生名单并批量删除。', path: '/admin/students' },
  { title: '数据删除（软删除）', desc: '删除未审核记录并进入彻底删除。', path: '/admin/soft-delete' },
  { title: '创建用户 / 邀请', desc: '新增用户并发送邀请邮件。', path: '/admin/users' },
  { title: '密码策略', desc: '设置学生密码校验规则。', path: '/admin/password-policy' },
  { title: '认证重置', desc: '发送非学生 TOTP/Passkey 重置链接。', path: '/admin/auth-reset' },
  { title: '一次性重置码', desc: '生成学生密码/认证重置码。', path: '/admin/reset-code' },
]

const go = (path: string) => {
  void router.push(path)
}
</script>

<template>
  <section class="hero">
    <h1>管理台</h1>
    <p>请选择需要的管理功能。</p>
  </section>

  <div class="card-grid">
    <el-card v-for="section in sections" :key="section.path" class="card">
      <h3>{{ section.title }}</h3>
      <p>{{ section.desc }}</p>
      <el-button type="primary" @click="go(section.path)">进入</el-button>
    </el-card>
  </div>
</template>
