<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import QRCode from 'qrcode'
import { bootstrapAdmin, bootstrapStatus, totpEnrollFinish, totpEnrollStart } from '../api/auth'
import { useRequest } from '../composables/useRequest'

const router = useRouter()
const { loading: requestLoading, error: requestError, run } = useRequest()
const { error: statusError, run: runStatus } = useRequest()
const { loading: totpLoading, error: totpError, run: runTotp } = useRequest()
const formRef = ref()
const totpFormRef = ref()
const status = ref<boolean | null>(null)
const step = ref<'bootstrap' | 'totp'>('bootstrap')
const qrDataUrl = ref('')

const form = reactive({
  username: '',
  display_name: '',
  token: '',
})

const totpForm = reactive({
  enrollment_id: '',
  otpauth_url: '',
  secret: '',
  code: '',
})

const rules = {
  username: [{ required: true, message: '请输入管理员账号', trigger: 'blur' }],
  display_name: [{ required: true, message: '请输入管理员名称', trigger: 'blur' }],
}

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

const startTotpEnrollment = async () => {
  const data = await totpEnrollStart({ device_label: '初始化认证器' })
  totpForm.enrollment_id = data.enrollment_id
  totpForm.otpauth_url = data.otpauth_url
  totpForm.secret = extractSecret(data.otpauth_url)
  totpForm.code = ''
  qrDataUrl.value = await QRCode.toDataURL(data.otpauth_url, { margin: 1, width: 240 })
  step.value = 'totp'
}

const loadStatus = async () => {
  await runStatus(
    async () => {
      const data = await bootstrapStatus()
      status.value = data.ready
      if (data.ready) {
        await router.replace('/login')
        return
      }
      if (data.needs_totp) {
        await startTotpEnrollment()
      }
    },
    { silent: true },
  )
}

onMounted(() => {
  void loadStatus()
})

const handleBootstrap = async () => {
  if (!formRef.value) return
  await formRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await run(async () => {
      const payload = {
        username: form.username,
        display_name: form.display_name,
        token: form.token.trim() || undefined,
      }
      await bootstrapAdmin(payload)
      await startTotpEnrollment()
    }, { successMessage: '管理员已创建，请绑定 TOTP' })
  })
}

const handleTotpFinish = async () => {
  if (!totpFormRef.value) return
  await totpFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await runTotp(async () => {
      await totpEnrollFinish({
        enrollment_id: totpForm.enrollment_id,
        code: totpForm.code.trim(),
      })
      await router.replace('/login')
    }, { successMessage: 'TOTP 绑定完成，请登录' })
  })
}
</script>

<template>
  <section class="hero">
    <h1>设置向导</h1>
    <p v-if="step === 'bootstrap'">首次使用请创建管理员账号，并绑定 TOTP。</p>
    <p v-else>请绑定 TOTP 认证器，完成后进入登录页面。</p>
  </section>

  <el-card class="card">
    <el-form
      v-if="step === 'bootstrap'"
      ref="formRef"
      :model="form"
      :rules="rules"
      label-position="top"
    >
      <el-form-item label="管理员账号" prop="username">
        <el-input v-model="form.username" placeholder="建议使用工号或管理员账号" />
      </el-form-item>
      <el-form-item label="管理员名称" prop="display_name">
        <el-input v-model="form.display_name" placeholder="显示名称" />
      </el-form-item>
      <el-form-item label="引导令牌（可选）">
        <el-input v-model="form.token" placeholder="如后端配置了引导令牌，请填写" />
      </el-form-item>
      <el-button type="primary" :loading="requestLoading" @click="handleBootstrap">
        创建管理员并继续
      </el-button>
    </el-form>
    <el-form
      v-else
      ref="totpFormRef"
      :model="totpForm"
      :rules="totpRules"
      label-position="top"
    >
      <el-form-item v-if="qrDataUrl" label="二维码">
        <img :src="qrDataUrl" alt="TOTP 二维码" style="max-width: 240px; width: 100%" />
      </el-form-item>
      <el-form-item label="TOTP 密钥">
        <el-input v-model="totpForm.secret" readonly />
      </el-form-item>
      <el-form-item label="otpauth URL">
        <el-input v-model="totpForm.otpauth_url" readonly />
      </el-form-item>
      <el-form-item label="验证码" prop="code">
        <el-input v-model="totpForm.code" placeholder="6 位数字" />
      </el-form-item>
      <el-button type="primary" :loading="totpLoading" @click="handleTotpFinish">
        完成绑定并进入登录
      </el-button>
    </el-form>
    <el-alert
      v-if="requestError || statusError || totpError"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="requestError || statusError || totpError"
      :closable="false"
    />
  </el-card>
</template>
