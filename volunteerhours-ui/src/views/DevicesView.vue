<script setup lang="ts">
import { ref } from 'vue'
import { listDevices, regenerateRecoveryCodes } from '../api/auth'
import { useRequest } from '../composables/useRequest'

const devices = ref('')
const codes = ref('')
const devicesRequest = useRequest()
const codesRequest = useRequest()

const handleLoad = async () => {
  await devicesRequest.run(
    async () => {
      const data = await listDevices()
      devices.value = JSON.stringify(data, null, 2)
    },
    { successMessage: '已刷新设备列表' },
  )
}

const handleRegenerate = async () => {
  await codesRequest.run(
    async () => {
      const data = await regenerateRecoveryCodes()
      codes.value = data.codes.join('\n')
    },
    { successMessage: '恢复码已生成' },
  )
}
</script>

<template>
  <section class="hero">
    <h1>设备与恢复码</h1>
    <p>管理 Passkey/TOTP 设备，生成恢复码。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>已绑定设备</h3>
      <el-button :loading="devicesRequest.loading" @click="handleLoad">刷新列表</el-button>
      <pre v-if="devices">{{ devices }}</pre>
    </el-card>
    <el-card class="card">
      <h3>恢复码</h3>
      <el-button type="primary" :loading="codesRequest.loading" @click="handleRegenerate">
        生成恢复码
      </el-button>
      <pre v-if="codes">{{ codes }}</pre>
    </el-card>
  </div>

  <el-alert
    v-if="devicesRequest.error || codesRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="devicesRequest.error || codesRequest.error"
    :closable="false"
  />
</template>
