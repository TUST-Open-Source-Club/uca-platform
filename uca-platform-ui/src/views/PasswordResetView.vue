<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { getPasswordPolicy, passwordResetConfirm, type PasswordPolicy } from '../api/auth'
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

const passwordPolicy = ref<PasswordPolicy | null>(null)
const passwordHint = computed(() => {
  if (!passwordPolicy.value) return '密码规则加载中...'
  const parts = [`至少 ${passwordPolicy.value.min_length} 位`]
  if (passwordPolicy.value.require_uppercase) parts.push('包含大写字母')
  if (passwordPolicy.value.require_lowercase) parts.push('包含小写字母')
  if (passwordPolicy.value.require_digit) parts.push('包含数字')
  if (passwordPolicy.value.require_symbol) parts.push('包含特殊符号')
  return `密码规则：${parts.join('，')}`
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

onMounted(async () => {
  try {
    passwordPolicy.value = await getPasswordPolicy()
  } catch {
    passwordPolicy.value = null
  }
})
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
      <el-alert
        type="info"
        show-icon
        :title="passwordHint"
        :closable="false"
        style="margin-bottom: 12px"
      />
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
