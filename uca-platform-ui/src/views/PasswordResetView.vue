<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { passwordResetConfirm } from '../api/auth'
import { useRequest } from '../composables/useRequest'

const route = useRoute()
const router = useRouter()
const formRef = ref()
const request = useRequest()

const token = computed(() => (route.query.token as string | undefined) ?? '')

const form = reactive({
  new_password: '',
  confirm_password: '',
})

const rules = {
  new_password: [{ required: true, message: '请输入新密码', trigger: 'blur' }],
  confirm_password: [
    {
      required: true,
      message: '请确认新密码',
      trigger: 'blur',
    },
    {
      validator: (_rule: unknown, value: string, callback: (err?: Error) => void) => {
        if (value !== form.new_password) {
          callback(new Error('两次输入的密码不一致'))
          return
        }
        callback()
      },
      trigger: 'blur',
    },
  ],
}

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await request.run(async () => {
      if (!token.value) {
        throw new Error('缺少重置令牌')
      }
      await passwordResetConfirm({ token: token.value, new_password: form.new_password })
      await router.replace('/login')
    }, { successMessage: '密码已重置，请登录' })
  })
}
</script>

<template>
  <section class="hero">
    <h1>设置新密码</h1>
    <p>请设置新的登录密码。</p>
  </section>

  <el-card class="card">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="新密码" prop="new_password">
        <el-input v-model="form.new_password" type="password" show-password />
      </el-form-item>
      <el-form-item label="确认密码" prop="confirm_password">
        <el-input v-model="form.confirm_password" type="password" show-password />
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleSubmit">
        提交
      </el-button>
    </el-form>
    <el-alert
      v-if="request.error"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="request.error"
      :closable="false"
    />
  </el-card>
</template>
