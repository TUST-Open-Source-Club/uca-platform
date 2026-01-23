<script setup lang="ts">
import { reactive, ref } from 'vue'
import { generateResetCode } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const resetCodeForm = reactive({
  username: '',
  purpose: 'password',
})

const result = ref<{ key: string; value: string }[] | null>(null)
const resetCodeRequest = useRequest()

const handleResetCode = async () => {
  await resetCodeRequest.run(async () => {
    const data = await generateResetCode({
      username: resetCodeForm.username,
      purpose: resetCodeForm.purpose as 'password' | 'totp' | 'passkey',
    })
    result.value = [
      { key: 'code', value: data.code ?? '-' },
      { key: 'expires_in_minutes', value: String(data.expires_in_minutes ?? '-') },
    ]
  }, { successMessage: '重置码已生成' })
}
</script>

<template>
  <section class="hero">
    <h1>一次性重置码</h1>
    <p>生成学生密码或认证重置码。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <el-form :model="resetCodeForm" label-position="top">
        <el-form-item label="用户名">
          <el-input v-model="resetCodeForm.username" placeholder="学号或工号" />
        </el-form-item>
        <el-form-item label="重置类型">
          <el-select v-model="resetCodeForm.purpose">
            <el-option label="学生密码" value="password" />
            <el-option label="TOTP" value="totp" />
            <el-option label="Passkey" value="passkey" />
          </el-select>
        </el-form-item>
        <el-button type="primary" :loading="resetCodeRequest.loading" @click="handleResetCode">
          生成重置码
        </el-button>
      </el-form>
      <p style="margin-top: 8px; color: var(--muted)">重置码 24 小时内有效，仅可使用一次。</p>
    </el-card>
  </div>

  <el-alert
    v-if="resetCodeRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="resetCodeRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <el-table :data="result" border>
      <el-table-column prop="key" label="字段" width="200" />
      <el-table-column prop="value" label="结果" />
    </el-table>
  </el-card>
</template>
