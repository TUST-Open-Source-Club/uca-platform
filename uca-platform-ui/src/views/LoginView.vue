<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { loginOptions, passkeyFinish, passkeyStart, passwordLogin, totpVerify } from '../api/auth'
import { listCompetitionsPublic } from '../api/catalog'
import { useRequest } from '../composables/useRequest'
import { useAuthStore } from '../stores/auth'
import { credentialToJson, normalizeRequestOptions } from '../utils/webauthn'

const formRef = ref()
const result = ref('')
const router = useRouter()
const { loading: requestLoading, error: requestError, run: runRequest } = useRequest()
const { run: runOptions } = useRequest()
const { loading: competitionsLoading, error: competitionsError, run: runCompetitions } = useRequest()
const competitions = ref<{ id: string; name: string }[]>([])

const form = reactive({
  username: '',
  method: 'passkey',
  code: '',
  password: '',
})

const authStore = useAuthStore()
const methods = [
  { id: 'passkey', title: 'Passkey 登录', desc: '使用设备生物识别或安全密钥' },
  { id: 'totp', title: 'TOTP 登录', desc: '输入动态验证码' },
  { id: 'password', title: '密码登录', desc: '仅学生可使用默认或自设密码' },
]
const availableMethods = ref<string[]>(['passkey', 'totp'])

const rules = {
  username: [{ required: true, message: '请输入学号/工号', trigger: 'blur' }],
  code: [
    {
      required: () => form.method === 'totp',
      message: '请输入验证码',
      trigger: 'blur',
    },
  ],
  password: [
    {
      required: () => form.method === 'password',
      message: '请输入密码',
      trigger: 'blur',
    },
  ],
}

const loadCompetitions = async () => {
  await runCompetitions(
    async () => {
      competitions.value = await listCompetitionsPublic()
    },
    { silent: true },
  )
}

onMounted(() => {
  void loadCompetitions()
})

const loadLoginOptions = async () => {
  if (!form.username) return
  await runOptions(
    async () => {
      const data = await loginOptions(form.username)
      availableMethods.value = data.methods
      if (!availableMethods.value.includes(form.method)) {
        form.method = data.methods[0] ?? 'passkey'
      }
    },
    { silent: true },
  )
}

const handleLogin = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await runRequest(
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

      if (form.method === 'password') {
        const data = await passwordLogin(form.username, form.password)
        result.value = JSON.stringify(data, null, 2)
        const profile = await authStore.refreshSession()
        if (!profile) {
          throw new Error('登录会话未建立，请检查 Cookie 或后端状态')
        }
        await router.push(authStore.homePath())
        return
      }

    },
      { successMessage: '登录成功' },
    )
  })
}
</script>

<template>
  <section class="hero">
    <h1>欢迎进入劳动教育学时认定系统</h1>
    <p>请使用 Passkey 或 TOTP 完成认证。学生可使用密码登录。</p>
  </section>

  <el-card class="card" style="margin-top: 16px">
    <h3>竞赛清单（仅查询）</h3>
    
    <el-table v-if="competitions.length" :data="competitions">
      <el-table-column prop="name" label="竞赛名称" />
    </el-table>
    <el-empty v-else :description="competitionsLoading ? '加载中' : '暂无竞赛数据'" />
    <el-alert
      v-if="competitionsError"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="competitionsError"
      :closable="false"
    />
  </el-card>

  <div class="card-grid">
    <el-card
      v-for="methodItem in methods.filter((item) => availableMethods.includes(item.id))"
      :key="methodItem.id"
      class="card"
    >
      <h3>{{ methodItem.title }}</h3>
      <p>{{ methodItem.desc }}</p>
      <el-button type="primary" @click="form.method = methodItem.id">选择</el-button>
    </el-card>
  </div>

  <el-card class="card" style="margin-top: 24px">
    <h3>账户输入</h3>
    <el-form ref="formRef" :model="form" :rules="rules" label-position="top">
      <el-form-item label="学号 / 工号" prop="username">
        <el-input v-model="form.username" placeholder="请输入学号或工号" @blur="loadLoginOptions" />
      </el-form-item>
      <el-form-item label="认证方式">
        <el-select v-model="form.method">
          <el-option
            v-for="method in methods.filter((item) => availableMethods.includes(item.id))"
            :key="method.id"
            :value="method.id"
            :label="method.title"
          />
        </el-select>
      </el-form-item>
      <el-form-item v-if="form.method === 'totp'" label="验证码" prop="code">
        <el-input v-model="form.code" placeholder="请输入验证码" />
      </el-form-item>
      <el-form-item v-if="form.method === 'password'" label="密码" prop="password">
        <el-input v-model="form.password" type="password" show-password placeholder="请输入密码" />
      </el-form-item>
      <el-button type="primary" :loading="requestLoading" @click="handleLogin">进入认证</el-button>
    </el-form>
    <p style="margin-top: 12px">
      <router-link to="/password-reset/request">学生忘记密码？</router-link>
    </p>
    <p>
      <router-link to="/reset-code">使用一次性重置码</router-link>
    </p>
    <pre v-if="result">{{ result }}</pre>
  </el-card>

  <el-alert
    v-if="requestError"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="requestError"
    :closable="false"
  />
</template>
