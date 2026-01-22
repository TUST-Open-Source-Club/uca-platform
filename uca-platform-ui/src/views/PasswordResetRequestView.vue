<script setup lang="ts">
import { reactive, ref } from 'vue'
import { passwordResetRequest } from '../api/auth'
import { useRequest } from '../composables/useRequest'

const formRef = ref()
const request = useRequest()

const form = reactive({
  username: '',
})

const rules = {
  username: [{ required: true, message: '请输入学号', trigger: 'blur' }],
}

const handleSubmit = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await request.run(async () => {
      await passwordResetRequest(form.username)
    }, { successMessage: '重置邮件已发送，请查收邮箱' })
  })
}
</script>

<template>
  <section class="hero">
    <h1>学生密码重置</h1>
    <p>请输入学号，系统将发送重置邮件到已绑定邮箱。</p>
  </section>

  <el-card class="card">
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="学号" prop="username">
        <el-input v-model="form.username" placeholder="请输入学号" />
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleSubmit">
        发送重置邮件
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
