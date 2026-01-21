<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { totpVerify } from '../api/auth'
import { useRequest } from '../composables/useRequest'
import { useAuthStore } from '../stores/auth'

const formRef = ref()
const result = ref('')
const form = reactive({ username: '', code: '' })
const request = useRequest()
const authStore = useAuthStore()
const router = useRouter()

const rules = {
  username: [{ required: true, message: '请输入学号/工号', trigger: 'blur' }],
  code: [{ required: true, message: '请输入验证码', trigger: 'blur' }],
}

const handleVerify = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await request.run(
      async () => {
        const data = await totpVerify(form.username, form.code)
        result.value = JSON.stringify(data, null, 2)
        const profile = await authStore.refreshSession()
        if (!profile) {
          throw new Error('登录会话未建立，请检查 Cookie 或后端状态')
        }
        await router.push(authStore.homePath())
      },
      { successMessage: '验证成功' },
    )
  })
}
</script>

<template>
  <section class="hero">
    <h1>二次验证</h1>
    <p>输入 TOTP 动态码以完成登录。</p>
  </section>

  <el-card class="card">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="学号 / 工号" prop="username">
        <el-input v-model="form.username" placeholder="请输入学号或工号" />
      </el-form-item>
      <el-form-item label="验证码" prop="code">
        <el-input v-model="form.code" placeholder="6 位动态码" />
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleVerify">验证并进入</el-button>
    </el-form>
    <pre v-if="result">{{ result }}</pre>
  </el-card>

  <el-alert
    v-if="request.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="request.error"
    :closable="false"
  />
</template>
