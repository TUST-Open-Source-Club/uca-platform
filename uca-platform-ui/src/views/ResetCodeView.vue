<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { resetStatus } from '../api/auth'
import { useRequest } from '../composables/useRequest'

const router = useRouter()
const formRef = ref()
const request = useRequest()

const form = reactive({
  code: '',
})

const rules = {
  code: [{ required: true, message: '请输入一次性重置码', trigger: 'blur' }],
}

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await request.run(async () => {
      const status = await resetStatus(form.code)
      if (!status.valid || !status.purpose) {
        throw new Error('重置码无效或已过期')
      }
      if (status.purpose === 'password') {
        await router.replace({ path: '/password-reset', query: { token: form.code } })
        return
      }
      await router.replace({ path: '/reset', query: { token: form.code } })
    })
  })
}
</script>

<template>
  <section class="hero">
    <h1>输入重置码</h1>
    <p>请输入管理员提供的一次性重置码。</p>
  </section>

  <el-card class="card">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="重置码" prop="code">
        <el-input v-model="form.code" placeholder="请输入重置码" />
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleSubmit">继续</el-button>
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
