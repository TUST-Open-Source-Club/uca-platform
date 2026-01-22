<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { listCompetitionsPublic } from '../api/catalog'
import { createContest, createVolunteer } from '../api/records'
import { bindEmail, changePassword, getPasswordPolicy, type PasswordPolicy } from '../api/auth'
import { listFormFieldsByType, type FormField } from '../api/forms'
import { useRequest } from '../composables/useRequest'

const volunteerFormRef = ref()
const contestFormRef = ref()
const result = ref('')
const volunteerRequest = useRequest()
const contestRequest = useRequest()
const fieldRequest = useRequest()
const accountRequest = useRequest()

const volunteerFields = ref<FormField[]>([])
const contestFields = ref<FormField[]>([])
const competitions = ref<{ id: string; name: string }[]>([])

const volunteerForm = reactive<Record<string, string | number>>({
  title: '',
  self_hours: 0,
  description: '',
})

const contestForm = reactive<Record<string, string | number>>({
  contest_name: '',
  award_level: '',
  self_hours: 0,
})

const accountFormRef = ref()
const accountForm = reactive({
  email: '',
  current_password: '',
  new_password: '',
})

const accountRules = {
  email: [{ required: true, message: '请输入邮箱', trigger: 'blur' }],
  current_password: [{ required: true, message: '请输入当前密码', trigger: 'blur' }],
  new_password: [{ required: true, message: '请输入新密码', trigger: 'blur' }],
}

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

const volunteerRules = reactive<Record<string, unknown>>({
  title: [{ required: true, message: '请输入标题', trigger: 'blur' }],
  self_hours: [
    { required: true, message: '请输入学时', trigger: 'change' },
    { validator: validateHours, trigger: 'change' },
  ],
})

const contestRules = reactive<Record<string, unknown>>({
  contest_name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
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
    const [volunteer, contest, competitionList] = await Promise.all([
      listFormFieldsByType('volunteer'),
      listFormFieldsByType('contest'),
      listCompetitionsPublic(),
    ])
    volunteerFields.value = volunteer
    contestFields.value = contest
    competitions.value = competitionList
    applyFieldDefaults(volunteerForm, volunteer)
    applyFieldDefaults(contestForm, contest)
    applyFieldRules(volunteerRules, volunteer)
    applyFieldRules(contestRules, contest)
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
})

const handleVolunteerSubmit = async () => {
  if (!volunteerFormRef.value) return
  await volunteerFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await volunteerRequest.run(
      async () => {
        const data = await createVolunteer({
          title: String(volunteerForm.title),
          description: String(volunteerForm.description),
          self_hours: Number(volunteerForm.self_hours),
          custom_fields: extractCustomFields(volunteerFields.value, volunteerForm),
        })
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已提交志愿服务' },
    )
  })
}

const handleContestSubmit = async () => {
  if (!contestFormRef.value) return
  await contestFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    result.value = ''
    await contestRequest.run(
      async () => {
        const data = await createContest({
          contest_name: String(contestForm.contest_name),
          award_level: String(contestForm.award_level),
          self_hours: Number(contestForm.self_hours),
          custom_fields: extractCustomFields(contestFields.value, contestForm),
        })
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已提交竞赛获奖' },
    )
  })
}

const handleBindEmail = async () => {
  if (!accountFormRef.value) return
  await accountFormRef.value.validateField('email', async (valid: boolean) => {
    if (!valid) return
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
    }, { successMessage: '密码已更新' })
  })
}
</script>

<template>
  <section class="hero">
    <h1>学生中心</h1>
    <p>提交志愿服务与竞赛获奖记录，跟踪审核进度。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>志愿服务填报</h3>
      <el-form ref="volunteerFormRef" :model="volunteerForm" :rules="volunteerRules" label-position="top">
        <el-form-item label="标题" prop="title">
          <el-input v-model="volunteerForm.title" placeholder="活动名称" />
        </el-form-item>
        <el-form-item label="自评学时" prop="self_hours">
          <el-input-number v-model="volunteerForm.self_hours" :min="0" />
        </el-form-item>
        <el-form-item label="说明">
          <el-input v-model="volunteerForm.description" type="textarea" rows="3" />
        </el-form-item>
        <el-form-item
          v-for="field in volunteerFields"
          :key="field.id"
          :label="field.label"
          :prop="field.field_key"
        >
          <el-input
            v-if="field.field_type !== 'number'"
            v-model="volunteerForm[field.field_key]"
            :placeholder="field.label"
          />
          <el-input-number
            v-else
            v-model="volunteerForm[field.field_key]"
            :min="0"
          />
        </el-form-item>
        <el-button type="primary" :loading="volunteerRequest.loading" @click="handleVolunteerSubmit">
          提交
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛获奖填报</h3>
      <el-form ref="contestFormRef" :model="contestForm" :rules="contestRules" label-position="top">
        <el-form-item label="竞赛名称" prop="contest_name">
          <el-select v-model="contestForm.contest_name" filterable placeholder="请选择竞赛名称">
            <el-option v-for="item in competitions" :key="item.id" :label="item.name" :value="item.name" />
          </el-select>
        </el-form-item>
        <el-form-item label="获奖等级" prop="award_level">
          <el-input v-model="contestForm.award_level" placeholder="例如 省赛一等奖" />
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
    v-if="volunteerRequest.error || contestRequest.error || fieldRequest.error || accountRequest.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="volunteerRequest.error || contestRequest.error || fieldRequest.error || accountRequest.error"
    :closable="false"
  />
</template>
