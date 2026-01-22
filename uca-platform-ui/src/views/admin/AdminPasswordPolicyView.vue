<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { getPasswordPolicy, updatePasswordPolicy } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const passwordPolicyForm = reactive({
  min_length: 8,
  require_uppercase: false,
  require_lowercase: false,
  require_digit: true,
  require_symbol: false,
})

const result = ref('')
const policyRequest = useRequest()

const loadPasswordPolicy = async () => {
  await policyRequest.run(async () => {
    const data = await getPasswordPolicy()
    passwordPolicyForm.min_length = data.min_length
    passwordPolicyForm.require_uppercase = data.require_uppercase
    passwordPolicyForm.require_lowercase = data.require_lowercase
    passwordPolicyForm.require_digit = data.require_digit
    passwordPolicyForm.require_symbol = data.require_symbol
  }, { silent: true })
}

const handleUpdatePolicy = async () => {
  await policyRequest.run(async () => {
    const data = await updatePasswordPolicy({
      min_length: passwordPolicyForm.min_length,
      require_uppercase: passwordPolicyForm.require_uppercase,
      require_lowercase: passwordPolicyForm.require_lowercase,
      require_digit: passwordPolicyForm.require_digit,
      require_symbol: passwordPolicyForm.require_symbol,
    })
    result.value = JSON.stringify(data, null, 2)
  }, { successMessage: '密码策略已更新' })
}

onMounted(() => {
  void loadPasswordPolicy()
})
</script>

<template>
  <section class="hero">
    <h1>密码策略</h1>
    <p>设置学生密码校验规则。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <el-form :model="passwordPolicyForm" label-position="top">
        <el-form-item label="最小长度">
          <el-input-number v-model="passwordPolicyForm.min_length" :min="4" :max="64" />
        </el-form-item>
        <el-form-item label="包含大写字母">
          <el-select v-model="passwordPolicyForm.require_uppercase">
            <el-option label="否" :value="false" />
            <el-option label="是" :value="true" />
          </el-select>
        </el-form-item>
        <el-form-item label="包含小写字母">
          <el-select v-model="passwordPolicyForm.require_lowercase">
            <el-option label="否" :value="false" />
            <el-option label="是" :value="true" />
          </el-select>
        </el-form-item>
        <el-form-item label="包含数字">
          <el-select v-model="passwordPolicyForm.require_digit">
            <el-option label="否" :value="false" />
            <el-option label="是" :value="true" />
          </el-select>
        </el-form-item>
        <el-form-item label="包含特殊符号">
          <el-select v-model="passwordPolicyForm.require_symbol">
            <el-option label="否" :value="false" />
            <el-option label="是" :value="true" />
          </el-select>
        </el-form-item>
        <el-button type="primary" :loading="policyRequest.loading" @click="handleUpdatePolicy">
          保存策略
        </el-button>
      </el-form>
    </el-card>
  </div>

  <el-alert
    v-if="policyRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="policyRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
