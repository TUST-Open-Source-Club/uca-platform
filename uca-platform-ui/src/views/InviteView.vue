<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useRoute } from 'vue-router'
import QRCode from 'qrcode'
import {
  inviteAccept,
  inviteStatus,
  passkeyRegisterFinish,
  passkeyRegisterStart,
  totpEnrollFinish,
  totpEnrollStart,
} from '../api/auth'
import { useRequest } from '../composables/useRequest'
import { normalizeCreationOptions, registrationCredentialToJson } from '../utils/webauthn'

const route = useRoute()
const token = computed(() => (route.query.token as string | undefined) ?? '')
const info = ref<{
  email?: string
  username?: string
  display_name?: string
  role?: string
  valid: boolean
} | null>(null)
const accepted = ref(false)
const totpFormRef = ref()
const step = ref<'idle' | 'totp'>('idle')
const qrDataUrl = ref('')

const statusRequest = useRequest()
const acceptRequest = useRequest()
const totpRequest = useRequest()
const passkeyRequest = useRequest()

const totpForm = reactive({
  enrollment_id: '',
  otpauth_url: '',
  secret: '',
  code: '',
})

const totpRules = {
  code: [{ required: true, message: '请输入认证器验证码', trigger: 'blur' }],
}

const extractSecret = (url: string) => {
  try {
    const parsed = new URL(url)
    return parsed.searchParams.get('secret') ?? ''
  } catch {
    return ''
  }
}

const loadStatus = async () => {
  if (!token.value) {
    info.value = { valid: false }
    return
  }
  await statusRequest.run(
    async () => {
      info.value = await inviteStatus(token.value)
    },
    { silent: true },
  )
}

onMounted(() => {
  void loadStatus()
})

const handleAccept = async () => {
  if (!token.value) return
  await acceptRequest.run(
    async () => {
      await inviteAccept(token.value)
      accepted.value = true
    },
    { successMessage: '邀请已接受，请绑定 TOTP 或 Passkey' },
  )
}

const handleTotpStart = async () => {
  await totpRequest.run(async () => {
    const data = await totpEnrollStart({ device_label: '邀请注册' })
    totpForm.enrollment_id = data.enrollment_id
    totpForm.otpauth_url = data.otpauth_url
    totpForm.secret = extractSecret(data.otpauth_url)
    totpForm.code = ''
    qrDataUrl.value = await QRCode.toDataURL(data.otpauth_url, { margin: 1, width: 240 })
    step.value = 'totp'
  })
}

const handleTotpFinish = async () => {
  if (!totpFormRef.value) return
  await totpFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await totpRequest.run(async () => {
      await totpEnrollFinish({
        enrollment_id: totpForm.enrollment_id,
        code: totpForm.code.trim(),
      })
    }, { successMessage: 'TOTP 绑定完成，请返回登录页面' })
  })
}

const handlePasskeyRegister = async () => {
  const username = info.value?.username
  if (!username) {
    throw new Error('缺少用户名')
  }
  await passkeyRequest.run(async () => {
    if (!window.PublicKeyCredential) {
      throw new Error('当前浏览器不支持 Passkey')
    }
    const data = await passkeyRegisterStart(username)
    const options = normalizeCreationOptions(
      data.public_key as unknown as PublicKeyCredentialCreationOptions,
    )
    const credential = (await navigator.credentials.create({
      publicKey: options,
    })) as PublicKeyCredential | null
    if (!credential) {
      throw new Error('未获取到 Passkey 凭据')
    }
    await passkeyRegisterFinish(data.session_id, registrationCredentialToJson(credential))
  }, { successMessage: 'Passkey 已绑定，请返回登录页面' })
}
</script>

<template>
  <section class="hero">
    <h1>邀请注册</h1>
    <p>请完成账户激活，并绑定 TOTP 或 Passkey。</p>
  </section>

  <el-card class="card">
    <div v-if="info?.valid">
      <p>用户名：{{ info.username }}</p>
      <p>角色：{{ info.role }}</p>
      <p>邮箱：{{ info.email }}</p>
      <el-button type="primary" :loading="acceptRequest.loading" @click="handleAccept">
        接受邀请
      </el-button>
    </div>
    <el-empty v-else description="邀请无效或已过期" />
  </el-card>

  <el-card v-if="accepted" class="card" style="margin-top: 16px">
    <h3>绑定 TOTP</h3>
    <el-button :loading="totpRequest.loading" @click="handleTotpStart">开始绑定</el-button>
    <el-form
      v-if="step === 'totp'"
      ref="totpFormRef"
      :model="totpForm"
      :rules="totpRules"
      label-position="top"
      style="margin-top: 12px"
    >
      <el-form-item v-if="qrDataUrl" label="二维码">
        <img :src="qrDataUrl" alt="TOTP 二维码" style="max-width: 240px; width: 100%" />
      </el-form-item>
      <el-form-item label="密钥">
        <el-input v-model="totpForm.secret" readonly />
      </el-form-item>
      <el-form-item label="验证码" prop="code">
        <el-input v-model="totpForm.code" placeholder="6 位数字" />
      </el-form-item>
      <el-button type="primary" :loading="totpRequest.loading" @click="handleTotpFinish">
        完成 TOTP 绑定
      </el-button>
    </el-form>
  </el-card>

  <el-card v-if="accepted" class="card" style="margin-top: 16px">
    <h3>绑定 Passkey</h3>
    <el-button type="primary" :loading="passkeyRequest.loading" @click="handlePasskeyRegister">
      绑定 Passkey
    </el-button>
  </el-card>

  <el-alert
    v-if="statusRequest.error || acceptRequest.error || totpRequest.error || passkeyRequest.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="statusRequest.error || acceptRequest.error || totpRequest.error || passkeyRequest.error"
    :closable="false"
  />
</template>
