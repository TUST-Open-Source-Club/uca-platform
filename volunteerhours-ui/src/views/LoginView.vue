<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { passkeyFinish, passkeyStart, recoveryVerify, totpVerify } from '../api/auth'
import { listCompetitionsPublic } from '../api/catalog'
import { useRequest } from '../composables/useRequest'
import { useAuthStore } from '../stores/auth'
import { credentialToJson, normalizeRequestOptions } from '../utils/webauthn'

const formRef = ref()
const result = ref('')
const router = useRouter()
const request = useRequest()
const competitionsRequest = useRequest()
const competitions = ref<{ id: string; name: string }[]>([])

const form = reactive({
  username: '',
  method: 'passkey',
  code: '',
})

const authStore = useAuthStore()
const methods = [
  { id: 'passkey', title: 'Passkey 登录', desc: '使用设备生物识别或安全密钥' },
  { id: 'totp', title: 'TOTP 登录', desc: '输入动态验证码' },
  { id: 'recovery', title: '恢复码登录', desc: '使用一次性恢复码' },
]

const rules = {
  username: [{ required: true, message: '请输入学号/工号', trigger: 'blur' }],
  code: [
    {
      required: () => form.method !== 'passkey',
      message: '请输入验证码',
      trigger: 'blur',
    },
  ],
}

const loadCompetitions = async () => {
  await competitionsRequest.run(
    async () => {
      competitions.value = await listCompetitionsPublic()
    },
    { silent: true },
  )
}

onMounted(() => {
  void loadCompetitions()
})

const handleLogin = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await request.run(
      async () => {
      if (form.method === 'passkey') {
        if (!window.PublicKeyCredential) {
          throw new Error('当前浏览器不支持 Passkey')
        }
        const data = await passkeyStart(form.username)
        const options = normalizeRequestOptions(
          data.public_key as unknown as PublicKeyCredentialRequestOptions,
        )
        const credential = (await navigator.credentials.get({
          publicKey: options,
        })) as PublicKeyCredential | null
        if (!credential) {
          throw new Error('未获取到 Passkey 凭据')
        }
        const finish = await passkeyFinish(data.session_id, credentialToJson(credential))
        result.value = JSON.stringify(finish, null, 2)
        const profile = await authStore.refreshSession()
        if (!profile) {
          throw new Error('登录会话未建立，请检查 Cookie 或后端状态')
        }
        await router.push(authStore.homePath())
        return
      }

      if (form.method === 'totp') {
        const data = await totpVerify(form.username, form.code)
        result.value = JSON.stringify(data, null, 2)
        const profile = await authStore.refreshSession()
        if (!profile) {
          throw new Error('登录会话未建立，请检查 Cookie 或后端状态')
        }
        await router.push(authStore.homePath())
        return
      }

      const data = await recoveryVerify(form.username, form.code)
      result.value = JSON.stringify(data, null, 2)
      const profile = await authStore.refreshSession()
      if (!profile) {
        throw new Error('登录会话未建立，请检查 Cookie 或后端状态')
      }
      await router.push(authStore.homePath())
    },
      { successMessage: '登录成功' },
    )
  })
}
</script>

<template>
  <section class="hero">
    <h1>欢迎进入志愿时长与竞赛统计平台</h1>
    <p>请使用 Passkey 或 TOTP 完成认证。系统不支持纯密码登录。</p>
  </section>

  <el-card class="card" style="margin-top: 16px">
    <h3>竞赛清单（仅查询）</h3>
    <el-table v-if="competitions.length" :data="competitions">
      <el-table-column prop="name" label="竞赛名称" />
    </el-table>
    <el-empty v-else :description="competitionsRequest.loading ? '加载中' : '暂无竞赛数据'" />
    <el-alert
      v-if="competitionsRequest.error"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="competitionsRequest.error"
      :closable="false"
    />
  </el-card>

  <div class="card-grid">
    <el-card v-for="methodItem in methods" :key="methodItem.id" class="card">
      <h3>{{ methodItem.title }}</h3>
      <p>{{ methodItem.desc }}</p>
      <el-button type="primary" @click="form.method = methodItem.id">选择</el-button>
    </el-card>
  </div>

  <el-card class="card" style="margin-top: 24px">
    <h3>账户输入</h3>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="学号 / 工号" prop="username">
        <el-input v-model="form.username" placeholder="请输入学号或工号" />
      </el-form-item>
      <el-form-item label="认证方式">
        <el-select v-model="form.method">
          <el-option value="passkey" label="Passkey" />
          <el-option value="totp" label="TOTP" />
          <el-option value="recovery" label="恢复码" />
        </el-select>
      </el-form-item>
      <el-form-item v-if="form.method !== 'passkey'" label="验证码" prop="code">
        <el-input v-model="form.code" placeholder="请输入验证码" />
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleLogin">进入认证</el-button>
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
