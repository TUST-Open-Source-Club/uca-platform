<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import QRCode from 'qrcode'
import {
  deleteDevice,
  getCurrentUser,
  listDevices,
  passkeyRegisterFinish,
  passkeyRegisterStart,
  reauthPasskeyFinish,
  reauthPasskeyStart,
  reauthPassword,
  reauthTotp,
  totpEnrollFinish,
  totpEnrollStart,
} from '../api/auth'
import { useAuthStore } from '../stores/auth'
import { useRequest } from '../composables/useRequest'
import { credentialToJson, normalizeCreationOptions, normalizeRequestOptions, registrationCredentialToJson } from '../utils/webauthn'

type Device = {
  id: string
  device_type: string
  label: string
  credential_id?: string | null
  created_at?: string
  last_used_at?: string | null
}

const authStore = useAuthStore()
const devices = ref<Device[]>([])
const qrDataUrl = ref('')
const totpStep = ref<'idle' | 'setup'>('idle')
const reauthToken = ref('')
const reauthExpiresAt = ref<number | null>(null)

const devicesRequest = useRequest()
const reauthRequest = useRequest()
const totpRequest = useRequest()
const passkeyRequest = useRequest()
const deleteRequest = useRequest()

const reauthFormRef = ref()
const totpFormRef = ref()

const reauthForm = reactive({
  method: 'password',
  password: '',
  code: '',
})

const totpForm = reactive({
  enrollment_id: '',
  otpauth_url: '',
  secret: '',
  code: '',
})

const reauthRules = {
  password: [
    { required: () => reauthForm.method === 'password', message: '请输入当前密码', trigger: 'blur' },
  ],
  code: [
    { required: () => reauthForm.method === 'totp', message: '请输入验证码', trigger: 'blur' },
  ],
}

const totpRules = {
  code: [{ required: true, message: '请输入认证器验证码', trigger: 'blur' }],
}

const hasReauth = computed(() => {
  if (!reauthToken.value || !reauthExpiresAt.value) return false
  return Date.now() < reauthExpiresAt.value
})

const reauthHint = computed(() => {
  if (!hasReauth.value || !reauthExpiresAt.value) return '未完成二次验证'
  const remaining = Math.max(0, Math.floor((reauthExpiresAt.value - Date.now()) / 1000))
  return `已完成二次验证（剩余 ${remaining} 秒）`
})

const setReauthToken = (token: string, expiresIn: number) => {
  reauthToken.value = token
  reauthExpiresAt.value = Date.now() + expiresIn * 1000
}

const consumeReauthToken = () => {
  reauthToken.value = ''
  reauthExpiresAt.value = null
}

const requireReauth = () => {
  if (!hasReauth.value) {
    throw new Error('请先完成二次验证')
  }
}

const extractSecret = (url: string) => {
  try {
    const parsed = new URL(url)
    return parsed.searchParams.get('secret') ?? ''
  } catch {
    return ''
  }
}

const handleLoad = async () => {
  await devicesRequest.run(async () => {
    const data = await listDevices()
    devices.value = data as Device[]
  }, { successMessage: '已刷新设备列表' })
}

const handleReauth = async () => {
  if (!reauthFormRef.value) return
  await reauthFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await reauthRequest.run(async () => {
      if (reauthForm.method === 'password') {
        const data = await reauthPassword(reauthForm.password)
        setReauthToken(data.token, data.expires_in)
        return
      }
      if (reauthForm.method === 'totp') {
        const data = await reauthTotp(reauthForm.code.trim())
        setReauthToken(data.token, data.expires_in)
        return
      }
      if (!navigator.credentials) {
        throw new Error('当前浏览器不支持 Passkey')
      }
      const start = await reauthPasskeyStart()
      const options = normalizeRequestOptions(start.public_key as PublicKeyCredentialRequestOptions)
      const credential = await navigator.credentials.get({ publicKey: options })
      if (!credential) {
        throw new Error('Passkey 验证已取消')
      }
      const finish = await reauthPasskeyFinish(start.session_id, credentialToJson(credential as PublicKeyCredential))
      setReauthToken(finish.token, finish.expires_in)
    }, { successMessage: '二次验证完成' })
  })
}

const handleTotpStart = async () => {
  requireReauth()
  await totpRequest.run(async () => {
    const data = await totpEnrollStart({ device_label: '自助绑定' }, reauthToken.value)
    totpForm.enrollment_id = data.enrollment_id
    totpForm.otpauth_url = data.otpauth_url
    totpForm.secret = extractSecret(data.otpauth_url)
    totpForm.code = ''
    qrDataUrl.value = await QRCode.toDataURL(data.otpauth_url, { margin: 1, width: 240 })
    totpStep.value = 'setup'
  }, { successMessage: '已生成 TOTP 密钥' })
}

const handleTotpFinish = async () => {
  requireReauth()
  if (!totpFormRef.value) return
  await totpFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await totpRequest.run(async () => {
      await totpEnrollFinish({
        enrollment_id: totpForm.enrollment_id,
        code: totpForm.code.trim(),
      }, reauthToken.value)
      totpStep.value = 'idle'
      consumeReauthToken()
      await handleLoad()
    }, { successMessage: 'TOTP 已更新' })
  })
}

const handlePasskeyRegister = async () => {
  requireReauth()
  await passkeyRequest.run(async () => {
    if (!navigator.credentials) {
      throw new Error('当前浏览器不支持 Passkey')
    }
    const profile = authStore.user ?? (await getCurrentUser())
    if (!profile) {
      throw new Error('无法获取当前用户信息')
    }
    const start = await passkeyRegisterStart(profile.username)
    const options = normalizeCreationOptions(start.public_key as PublicKeyCredentialCreationOptions)
    const credential = await navigator.credentials.create({ publicKey: options })
    if (!credential) {
      throw new Error('Passkey 注册已取消')
    }
    await passkeyRegisterFinish(start.session_id, registrationCredentialToJson(credential as PublicKeyCredential), reauthToken.value)
    consumeReauthToken()
    await handleLoad()
  }, { successMessage: 'Passkey 已添加' })
}

const handleDeleteDevice = async (deviceId: string) => {
  requireReauth()
  await deleteRequest.run(async () => {
    await deleteDevice(deviceId, reauthToken.value)
    consumeReauthToken()
    await handleLoad()
  }, { successMessage: '设备已移除' })
}

onMounted(async () => {
  await authStore.ensureSession()
  await handleLoad()
})
</script>

<template>
  <section class="hero">
    <h1>设备与认证</h1>
    <p>添加或移除 Passkey/TOTP 设备，维护二次验证。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>二次验证</h3>
      <p>{{ reauthHint }}</p>
      <el-form ref="reauthFormRef" :model="reauthForm" :rules="reauthRules" label-position="top">
        <el-form-item label="验证方式">
          <el-select v-model="reauthForm.method">
            <el-option label="当前密码" value="password" />
            <el-option label="TOTP 验证码" value="totp" />
            <el-option label="Passkey" value="passkey" />
          </el-select>
        </el-form-item>
        <el-form-item v-if="reauthForm.method === 'password'" label="当前密码" prop="password">
          <el-input v-model="reauthForm.password" type="password" show-password />
        </el-form-item>
        <el-form-item v-if="reauthForm.method === 'totp'" label="验证码" prop="code">
          <el-input v-model="reauthForm.code" placeholder="6 位数字" />
        </el-form-item>
        <el-button type="primary" :loading="reauthRequest.loading" @click="handleReauth">
          完成二次验证
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>TOTP 认证器</h3>
      <p>需要先完成二次验证后才能更新认证器。</p>
      <el-button :loading="totpRequest.loading" @click="handleTotpStart">开始绑定</el-button>
      <el-form
        v-if="totpStep === 'setup'"
        ref="totpFormRef"
        :model="totpForm"
        :rules="totpRules"
        label-position="top"
        style="margin-top: 12px"
      >
        <el-form-item v-if="qrDataUrl" label="二维码">
          <img :src="qrDataUrl" alt="TOTP 二维码" style="max-width: 240px; width: 100%" />
        </el-form-item>
        <el-form-item label="TOTP 密钥">
          <el-input v-model="totpForm.secret" readonly />
        </el-form-item>
        <el-form-item label="验证码" prop="code">
          <el-input v-model="totpForm.code" placeholder="6 位数字" />
        </el-form-item>
        <el-button type="primary" :loading="totpRequest.loading" @click="handleTotpFinish">
          完成绑定
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>Passkey</h3>
      <p>需要先完成二次验证后才能新增或移除 Passkey。</p>
      <el-button type="primary" :loading="passkeyRequest.loading" @click="handlePasskeyRegister">
        新增 Passkey
      </el-button>
    </el-card>

    <el-card class="card">
      <h3>已绑定设备</h3>
      <el-button :loading="devicesRequest.loading" @click="handleLoad">刷新列表</el-button>
      <el-table v-if="devices.length" :data="devices" style="margin-top: 12px">
        <el-table-column prop="label" label="设备名称" />
        <el-table-column prop="device_type" label="类型" />
        <el-table-column prop="created_at" label="创建时间" />
        <el-table-column prop="last_used_at" label="最近使用" />
        <el-table-column label="操作" width="120">
          <template #default="{ row }">
            <el-button
              type="danger"
              size="small"
              :loading="deleteRequest.loading"
              @click="handleDeleteDevice(row.id)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
      <p v-else style="margin-top: 12px">暂无设备</p>
    </el-card>

  </div>

  <el-alert
    v-if="devicesRequest.error || reauthRequest.error || totpRequest.error || passkeyRequest.error || deleteRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="devicesRequest.error || reauthRequest.error || totpRequest.error || passkeyRequest.error || deleteRequest.error"
    :closable="false"
  />
</template>
