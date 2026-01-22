<script setup lang="ts">
import { reactive } from 'vue'
import { resetUserPasskey, resetUserTotp } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const resetForm = reactive({
  username: '',
  method: 'totp',
})

const resetRequest = useRequest()

const handleResetAuth = async () => {
  await resetRequest.run(async () => {
    if (resetForm.method === 'totp') {
      await resetUserTotp(resetForm.username)
    } else {
      await resetUserPasskey(resetForm.username)
    }
  }, { successMessage: '重置链接已发送' })
}
</script>

<template>
  <section class="hero">
    <h1>认证重置（非学生）</h1>
    <p>发送 TOTP 或 Passkey 重置链接。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <el-form :model="resetForm" label-position="top">
        <el-form-item label="用户名">
          <el-input v-model="resetForm.username" placeholder="工号" />
        </el-form-item>
        <el-form-item label="重置方式">
          <el-select v-model="resetForm.method">
            <el-option label="TOTP" value="totp" />
            <el-option label="Passkey" value="passkey" />
          </el-select>
        </el-form-item>
        <el-button type="primary" :loading="resetRequest.loading" @click="handleResetAuth">
          发送重置链接
        </el-button>
      </el-form>
    </el-card>
  </div>

  <el-alert
    v-if="resetRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="resetRequest.error"
    :closable="false"
  />
</template>
