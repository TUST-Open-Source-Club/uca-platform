<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import type { UploadFile } from 'element-plus'
import { listCompetitionsPublic, type CompetitionItem } from '../api/catalog'
import { uploadContestAttachment } from '../api/attachments'
import { createContest } from '../api/records'
import { bindEmail, changePassword, getPasswordPolicy, type PasswordPolicy } from '../api/auth'
import { listFormFieldsByType, type FormField } from '../api/forms'
import { getCurrentStudent, type StudentProfile } from '../api/students'
import { useRequest } from '../composables/useRequest'
import { useAuthStore } from '../stores/auth'

const contestFormRef = ref()
const result = ref('')
const contestRequest = useRequest()
const fieldRequest = useRequest()
const accountRequest = useRequest()
const studentRequest = useRequest()
const authStore = useAuthStore()

const contestFields = ref<FormField[]>([])
const competitions = ref<CompetitionItem[]>([])
const studentProfile = ref<StudentProfile | null>(null)

const contestForm = reactive<Record<string, string | number>>({
  contest_name: '',
  contest_year: 0,
  contest_category: '',
  contest_level: '',
  contest_role: '',
  award_level: '',
  award_date: '',
  self_hours: 0,
})
const attachmentFile = ref<File | null>(null)
const attachmentError = ref('')

const accountFormRef = ref()
const accountForm = reactive({
  email: '',
  current_password: '',
  new_password: '',
})

const accountRules = {
  email: [{ required: () => authStore.resetDelivery === 'email', message: '请输入邮箱', trigger: 'blur' }],
  current_password: [{ required: true, message: '请输入当前密码', trigger: 'blur' }],
  new_password: [{ required: true, message: '请输入新密码', trigger: 'blur' }],
}

const mustChangeDialog = ref(false)
watch(
  () => authStore.mustChangePassword,
  (value) => {
    mustChangeDialog.value = value
  },
  { immediate: true },
)

const passwordPolicy = ref<PasswordPolicy | null>(null)
const passwordHint = computed(() => {
  if (!passwordPolicy.value) return '密码规则加载中...'
  const parts = [`至少 ${passwordPolicy.value.min_length} 位`]
  if (passwordPolicy.value.require_uppercase) parts.push('包含大写字母')
  if (passwordPolicy.value.require_lowercase) parts.push('包含小写字母')
  if (passwordPolicy.value.require_digit) parts.push('包含数字')
  if (passwordPolicy.value.require_symbol) parts.push('包含特殊符号')
  return `密码规则：${parts.join('，')}`
})

const validateHours = (_: unknown, value: number, callback: (error?: Error) => void) => {
  if (Number(value) <= 0) {
    callback(new Error('学时需大于 0'))
    return
  }
  callback()
}

const contestRules = reactive<Record<string, unknown>>({
  contest_name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
  contest_level: [{ required: true, message: '请选择获奖级别', trigger: 'change' }],
  contest_role: [{ required: true, message: '请选择角色', trigger: 'change' }],
  award_level: [{ required: true, message: '请输入获奖等级', trigger: 'blur' }],
  self_hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
})

const applyFieldRules = (rules: Record<string, unknown>, fields: FormField[]) => {
  fields.forEach((field) => {
    if (field.required) {
      rules[field.field_key] = [{ required: true, message: `请输入${field.label}`, trigger: 'blur' }]
    }
  })
}

const applyFieldDefaults = (form: Record<string, string | number>, fields: FormField[]) => {
  fields.forEach((field) => {
    if (form[field.field_key] === undefined) {
      form[field.field_key] = ''
    }
  })
}

const applyStudentDefaults = (form: Record<string, string | number>, profile: StudentProfile | null) => {
  if (!profile) return
  const mapping: Record<string, string> = {
    student_no: profile.student_no,
    name: profile.name,
    gender: profile.gender,
    department: profile.department,
    major: profile.major,
    class_name: profile.class_name,
    phone: profile.phone,
  }
  Object.entries(mapping).forEach(([key, value]) => {
    if (form[key] === undefined || String(form[key]).trim() === '') {
      form[key] = value
    }
  })
}

const extractCustomFields = (fields: FormField[], form: Record<string, string | number>) => {
  const payload: Record<string, string> = {}
  fields.forEach((field) => {
    const value = form[field.field_key]
    if (value !== undefined && value !== null && String(value).trim() !== '') {
      payload[field.field_key] = String(value)
    }
  })
  return payload
}

const loadFields = async () => {
  await fieldRequest.run(async () => {
    const [contest, competitionList] = await Promise.all([
      listFormFieldsByType('contest'),
      listCompetitionsPublic(),
    ])
    contestFields.value = contest
    competitions.value = competitionList
    applyFieldDefaults(contestForm, contest)
    applyFieldRules(contestRules, contest)
  })
}

const loadStudentProfile = async () => {
  await studentRequest.run(async () => {
    studentProfile.value = await getCurrentStudent()
    applyStudentDefaults(contestForm, studentProfile.value)
  })
}

const loadPasswordPolicy = async () => {
  try {
    passwordPolicy.value = await getPasswordPolicy()
  } catch {
    passwordPolicy.value = null
  }
}

onMounted(() => {
  void loadFields()
  void loadPasswordPolicy()
  void loadStudentProfile()
})

watch(
  () => contestForm.contest_name,
  (value) => {
    if (!value) return
    const match = competitions.value.find((item) => item.name === String(value))
    if (!match) return
    if (match.year) {
      contestForm.contest_year = match.year
    }
    if (match.category) {
      contestForm.contest_category = match.category
    }
  },
)

const handleContestSubmit = async () => {
  if (!contestFormRef.value) return
  await contestFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    if (!attachmentFile.value) {
      attachmentError.value = '请上传获奖证明附件（PDF 或图片）'
      return
    }
    result.value = ''
    await contestRequest.run(
      async () => {
        const data = await createContest({
          contest_name: String(contestForm.contest_name),
          contest_year: contestForm.contest_year ? Number(contestForm.contest_year) : null,
          contest_category: contestForm.contest_category
            ? String(contestForm.contest_category)
            : null,
          contest_level: contestForm.contest_level ? String(contestForm.contest_level) : null,
          contest_role: contestForm.contest_role ? String(contestForm.contest_role) : null,
          award_level: String(contestForm.award_level),
          award_date: contestForm.award_date ? String(contestForm.award_date) : null,
          self_hours: Number(contestForm.self_hours),
          custom_fields: extractCustomFields(contestFields.value, contestForm),
        })
        const recordId = (data as { id?: string }).id
        if (!recordId) {
          throw new Error('记录创建失败，未返回记录 ID')
        }
        await uploadContestAttachment(recordId, attachmentFile.value as File)
        attachmentFile.value = null
        attachmentError.value = ''
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已提交竞赛获奖' },
    )
  })
}

const handleAttachmentChange = (file: UploadFile) => {
  attachmentFile.value = file.raw ?? null
  attachmentError.value = ''
}

const handleBindEmail = async () => {
  if (!accountFormRef.value) return
  await accountFormRef.value.validateField('email', async (valid: boolean) => {
    if (!valid) return
    if (!accountForm.email.trim()) {
      accountRequest.error = '请输入邮箱'
      return
    }
    await accountRequest.run(async () => {
      await bindEmail(accountForm.email)
    }, { successMessage: '邮箱已绑定' })
  })
}

const handleChangePassword = async () => {
  if (!accountFormRef.value) return
  await accountFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
  await accountRequest.run(async () => {
    await changePassword({
      current_password: accountForm.current_password,
      new_password: accountForm.new_password,
    })
    await authStore.refreshSession()
    mustChangeDialog.value = false
  }, { successMessage: '密码已更新' })
  })
}
</script>

<template>
  <section class="hero">
    <h1>学生中心</h1>
    <p>提交竞赛获奖与劳动教育学时认定申请。</p>
  </section>

  <el-dialog
    v-model="mustChangeDialog"
    title="首次登录请修改密码"
    width="520px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="false"
  >
    <el-form ref="accountFormRef" :model="accountForm" :rules="accountRules" label-position="top">
      <el-form-item label="当前密码" prop="current_password">
        <el-input v-model="accountForm.current_password" type="password" show-password />
      </el-form-item>
      <el-form-item label="新密码" prop="new_password">
        <el-input v-model="accountForm.new_password" type="password" show-password />
      </el-form-item>
      <el-alert
        type="info"
        show-icon
        :title="passwordHint"
        :closable="false"
        style="margin-bottom: 12px"
      />
      <el-button type="primary" :loading="accountRequest.loading" @click="handleChangePassword">
        修改密码并进入系统
      </el-button>
    </el-form>
  </el-dialog>

  <div v-if="!authStore.mustChangePassword" class="card-grid">
    <el-card class="card">
      <h3>竞赛获奖填报</h3>
      <el-form ref="contestFormRef" :model="contestForm" :rules="contestRules" label-position="top">
        <el-form-item label="竞赛名称" prop="contest_name">
          <el-select v-model="contestForm.contest_name" filterable placeholder="请选择竞赛名称">
            <el-option v-for="item in competitions" :key="item.id" :label="item.name" :value="item.name" />
          </el-select>
        </el-form-item>
        <el-form-item label="竞赛年份">
          <el-input-number v-model="contestForm.contest_year" :min="2000" :max="2100" />
        </el-form-item>
        <el-form-item label="竞赛类型（A/B）">
          <el-select v-model="contestForm.contest_category" placeholder="选择类型">
            <el-option label="A 类" value="A" />
            <el-option label="B 类" value="B" />
          </el-select>
        </el-form-item>
        <el-form-item label="获奖级别" prop="contest_level">
          <el-select v-model="contestForm.contest_level" placeholder="选择级别">
            <el-option label="国家级" value="国家级" />
            <el-option label="省级" value="省级" />
            <el-option label="校级" value="校级" />
          </el-select>
        </el-form-item>
        <el-form-item label="角色" prop="contest_role">
          <el-select v-model="contestForm.contest_role" placeholder="选择角色">
            <el-option label="负责人" value="负责人" />
            <el-option label="成员" value="成员" />
          </el-select>
        </el-form-item>
        <el-form-item label="获奖等级" prop="award_level">
          <el-input v-model="contestForm.award_level" placeholder="例如 省赛一等奖" />
        </el-form-item>
        <el-form-item label="获奖时间">
          <el-date-picker
            v-model="contestForm.award_date"
            type="date"
            value-format="YYYY-MM-DD"
            placeholder="选择日期"
          />
        </el-form-item>
        <el-form-item label="获奖证明附件">
          <el-upload
            :auto-upload="false"
            :limit="1"
            accept="application/pdf,image/*"
            :show-file-list="true"
            :on-change="handleAttachmentChange"
          >
            <el-button>选择附件</el-button>
          </el-upload>
          <div v-if="attachmentError" style="margin-top: 6px; color: var(--el-color-danger)">
            {{ attachmentError }}
          </div>
          <div style="margin-top: 6px; color: var(--muted); font-size: 12px">
            必须上传 PDF 或图片格式的获奖证明。
          </div>
        </el-form-item>
        <el-form-item label="自评学时" prop="self_hours">
          <el-input-number v-model="contestForm.self_hours" :min="0" />
        </el-form-item>
        <el-form-item
          v-for="field in contestFields"
          :key="field.id"
          :label="field.label"
          :prop="field.field_key"
        >
          <el-input
            v-if="field.field_type !== 'number'"
            v-model="contestForm[field.field_key]"
            :placeholder="field.label"
          />
          <el-input-number
            v-else
            v-model="contestForm[field.field_key]"
            :min="0"
          />
        </el-form-item>
        <el-button type="primary" :loading="contestRequest.loading" @click="handleContestSubmit">
          提交
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>账户安全</h3>
      <el-form ref="accountFormRef" :model="accountForm" :rules="accountRules" label-position="top">
        <el-form-item label="绑定邮箱" prop="email">
          <el-input v-model="accountForm.email" placeholder="用于重置密码" />
        </el-form-item>
        <div style="margin-bottom: 8px; color: var(--muted); font-size: 12px">
          纯内网部署可不绑定邮箱，外网部署建议绑定以便自助找回密码。
        </div>
        <el-button :loading="accountRequest.loading" @click="handleBindEmail">绑定邮箱</el-button>

        <el-divider style="margin: 16px 0" />

        <el-form-item label="当前密码" prop="current_password">
          <el-input v-model="accountForm.current_password" type="password" show-password />
        </el-form-item>
        <el-form-item label="新密码" prop="new_password">
          <el-input v-model="accountForm.new_password" type="password" show-password />
        </el-form-item>
        <el-alert
          type="info"
          show-icon
          :title="passwordHint"
          :closable="false"
          style="margin-bottom: 12px"
        />
        <el-button type="primary" :loading="accountRequest.loading" @click="handleChangePassword">
          修改密码
        </el-button>
      </el-form>
    </el-card>
  </div>

  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>

  <el-alert
    v-if="contestRequest.error || fieldRequest.error || accountRequest.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="contestRequest.error || fieldRequest.error || accountRequest.error"
    :closable="false"
  />
</template>
