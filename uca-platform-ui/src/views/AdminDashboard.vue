<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { UploadFile } from 'element-plus'
import {
  createCompetition,
  createUser,
  createFormField,
  deleteContestRecord,
  deleteStudent,
  deleteVolunteerRecord,
  generateResetCode,
  getPasswordPolicy,
  importCompetitions,
  importContestRecords,
  importVolunteerRecords,
  listCompetitions,
  listFormFields,
  resetUserPasskey,
  resetUserTotp,
  updatePasswordPolicy,
} from '../api/admin'
import { importStudents, queryStudents } from '../api/students'
import { queryContest, queryVolunteer } from '../api/records'
import { useRequest } from '../composables/useRequest'

const competitionFormRef = ref()
const formFieldRef = ref()
const importFormRef = ref()
const competitionImportRef = ref()
const volunteerImportRef = ref()
const contestImportRef = ref()
const userFormRef = ref()
const competitions = ref('')
const formFields = ref('')
const students = ref<any[]>([])
const volunteerRecords = ref<any[]>([])
const contestRecords = ref<any[]>([])
const importFile = ref<File | null>(null)
const competitionImportFile = ref<File | null>(null)
const volunteerImportFile = ref<File | null>(null)
const contestImportFile = ref<File | null>(null)
const result = ref('')
const competitionRequest = useRequest()
const formFieldRequest = useRequest()
const importRequest = useRequest()
const competitionImportRequest = useRequest()
const volunteerImportRequest = useRequest()
const contestImportRequest = useRequest()
const listRequest = useRequest()
const deleteRequest = useRequest()
const listDataRequest = useRequest()
const userRequest = useRequest()
const policyRequest = useRequest()
const resetRequest = useRequest()
const resetCodeRequest = useRequest()
const router = useRouter()

const competitionForm = reactive({
  name: '',
})

const formField = reactive({
  form_type: 'volunteer',
  field_key: '',
  label: '',
  field_type: 'text',
  required: false,
  order_index: 1,
})

const importForm = reactive({
  fileName: '',
})

const competitionImportForm = reactive({
  fileName: '',
})

const volunteerImportForm = reactive({
  fileName: '',
})

const contestImportForm = reactive({
  fileName: '',
})

const userForm = reactive({
  username: '',
  display_name: '',
  role: 'student',
  email: '',
})

const passwordPolicyForm = reactive({
  min_length: 8,
  require_uppercase: false,
  require_lowercase: false,
  require_digit: true,
  require_symbol: false,
})

const resetForm = reactive({
  username: '',
  method: 'totp',
})

const resetCodeForm = reactive({
  username: '',
  purpose: 'password',
})

const competitionRules = {
  name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
}

const formFieldRules = {
  field_key: [{ required: true, message: '请输入字段 Key', trigger: 'blur' }],
  label: [{ required: true, message: '请输入字段标签', trigger: 'blur' }],
  form_type: [{ required: true, message: '请选择表单类型', trigger: 'change' }],
  field_type: [{ required: true, message: '请选择字段类型', trigger: 'change' }],
  order_index: [{ required: true, message: '请输入排序序号', trigger: 'change' }],
}

const importRules = {
  fileName: [{ required: true, message: '请选择 Excel 文件', trigger: 'change' }],
}

const competitionImportRules = {
  fileName: [{ required: true, message: '请选择竞赛库 Excel 文件', trigger: 'change' }],
}

const volunteerImportRules = {
  fileName: [{ required: true, message: '请选择志愿服务导入文件', trigger: 'change' }],
}

const contestImportRules = {
  fileName: [{ required: true, message: '请选择竞赛获奖导入文件', trigger: 'change' }],
}

const userRules = {
  username: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  display_name: [{ required: true, message: '请输入显示名称', trigger: 'blur' }],
  role: [{ required: true, message: '请选择角色', trigger: 'change' }],
  email: [
    {
      validator: (_rule: unknown, value: string, callback: (err?: Error) => void) => {
        if (userForm.role !== 'student' && !value) {
          callback(new Error('非学生必须填写邮箱'))
          return
        }
        callback()
      },
      trigger: 'blur',
    },
  ],
}

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

onMounted(() => {
  void loadPasswordPolicy()
})

const loadCompetitions = async () => {
  await listRequest.run(async () => {
    const data = await listCompetitions()
    competitions.value = JSON.stringify(data, null, 2)
  })
}

const handleCompetitionCreate = async () => {
  if (!competitionFormRef.value) return
  await competitionFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await competitionRequest.run(
      async () => {
        const data = await createCompetition(competitionForm.name)
        result.value = JSON.stringify(data, null, 2)
        await loadCompetitions()
      },
      { successMessage: '已新增竞赛' },
    )
  })
}

const loadFormFields = async () => {
  await listRequest.run(async () => {
    const data = await listFormFields()
    formFields.value = JSON.stringify(data, null, 2)
  })
}

const handleFormFieldCreate = async () => {
  if (!formFieldRef.value) return
  await formFieldRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await formFieldRequest.run(
      async () => {
        const data = await createFormField(formField)
        result.value = JSON.stringify(data, null, 2)
        await loadFormFields()
      },
      { successMessage: '已新增字段' },
    )
  })
}

const handleImport = async () => {
  if (!importFormRef.value) return
  await importFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await importRequest.run(
      async () => {
        const data = await importStudents(importFile.value as File)
        result.value = JSON.stringify(data, null, 2)
      },
      { successMessage: '已上传学生名单' },
    )
  })
}

const handleFileChange = (file: UploadFile) => {
  importFile.value = file.raw ?? null
  importForm.fileName = file.name ?? ''
}

const handleCompetitionImport = async () => {
  if (!competitionImportRef.value) return
  await competitionImportRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await competitionImportRequest.run(async () => {
      const data = await importCompetitions(competitionImportFile.value as File)
      result.value = JSON.stringify(data, null, 2)
    }, { successMessage: '竞赛库已导入' })
  })
}

const handleCompetitionFileChange = (file: UploadFile) => {
  competitionImportFile.value = file.raw ?? null
  competitionImportForm.fileName = file.name ?? ''
}

const handleVolunteerImport = async () => {
  if (!volunteerImportRef.value) return
  await volunteerImportRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await volunteerImportRequest.run(async () => {
      const data = await importVolunteerRecords(volunteerImportFile.value as File)
      result.value = JSON.stringify(data, null, 2)
    }, { successMessage: '志愿服务记录已导入' })
  })
}

const handleVolunteerFileChange = (file: UploadFile) => {
  volunteerImportFile.value = file.raw ?? null
  volunteerImportForm.fileName = file.name ?? ''
}

const handleContestImport = async () => {
  if (!contestImportRef.value) return
  await contestImportRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await contestImportRequest.run(async () => {
      const data = await importContestRecords(contestImportFile.value as File)
      result.value = JSON.stringify(data, null, 2)
    }, { successMessage: '竞赛获奖记录已导入' })
  })
}

const handleContestFileChange = (file: UploadFile) => {
  contestImportFile.value = file.raw ?? null
  contestImportForm.fileName = file.name ?? ''
}

const loadDataLists = async () => {
  await listDataRequest.run(async () => {
    const [studentList, volunteerList, contestList] = await Promise.all([
      queryStudents({}),
      queryVolunteer(),
      queryContest(),
    ])
    students.value = studentList
    volunteerRecords.value = volunteerList
    contestRecords.value = contestList
  })
}

const handleDeleteStudent = async (studentNo: string) => {
  await deleteRequest.run(async () => {
    await deleteStudent(studentNo)
    await loadDataLists()
  }, { successMessage: '学生已删除' })
}

const handleDeleteVolunteerRecord = async (recordId: string) => {
  await deleteRequest.run(async () => {
    await deleteVolunteerRecord(recordId)
    await loadDataLists()
  }, { successMessage: '志愿记录已删除' })
}

const handleDeleteContestRecord = async (recordId: string) => {
  await deleteRequest.run(async () => {
    await deleteContestRecord(recordId)
    await loadDataLists()
  }, { successMessage: '竞赛记录已删除' })
}

const handleOpenPurge = async () => {
  await router.push('/purge')
}

const handleCreateUser = async () => {
  if (!userFormRef.value) return
  await userFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await userRequest.run(async () => {
      const payload = {
        username: userForm.username,
        display_name: userForm.display_name,
        role: userForm.role as 'student' | 'teacher' | 'reviewer' | 'admin',
        email: userForm.email || undefined,
      }
      const data = await createUser(payload)
      result.value = JSON.stringify(data, null, 2)
    }, { successMessage: '已提交用户创建' })
  })
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

const handleResetAuth = async () => {
  await resetRequest.run(async () => {
    if (resetForm.method === 'totp') {
      await resetUserTotp(resetForm.username)
    } else {
      await resetUserPasskey(resetForm.username)
    }
  }, { successMessage: '重置链接已发送' })
}

const handleResetCode = async () => {
  await resetCodeRequest.run(async () => {
    const data = await generateResetCode({
      username: resetCodeForm.username,
      purpose: resetCodeForm.purpose as 'password' | 'totp' | 'passkey',
    })
    result.value = JSON.stringify(data, null, 2)
  }, { successMessage: '重置码已生成' })
}
</script>

<template>
  <section class="hero">
    <h1>管理台</h1>
    <p>维护学生名单、竞赛名称库与模板配置。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>学生名单导入</h3>
      <el-form ref="importFormRef" :model="importForm" :rules="importRules" label-position="top">
        <el-form-item label="Excel 文件" prop="fileName">
          <el-upload
            :auto-upload="false"
            :limit="1"
            :show-file-list="true"
            :on-change="handleFileChange"
          >
            <el-button>选择文件</el-button>
          </el-upload>
        </el-form-item>
        <el-button
          type="primary"
          style="margin-top: 12px"
          :loading="importRequest.loading"
          @click="handleImport"
        >
          上传名单
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛名称库导入</h3>
      <el-form ref="competitionImportRef" :model="competitionImportForm" :rules="competitionImportRules" label-position="top">
        <el-form-item label="竞赛库 Excel" prop="fileName">
          <el-upload
            :auto-upload="false"
            :limit="1"
            :show-file-list="true"
            :on-change="handleCompetitionFileChange"
          >
            <el-button>选择文件</el-button>
          </el-upload>
        </el-form-item>
        <el-button
          type="primary"
          style="margin-top: 12px"
          :loading="competitionImportRequest.loading"
          @click="handleCompetitionImport"
        >
          导入竞赛库
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛名称库</h3>
      <el-form ref="competitionFormRef" :model="competitionForm" :rules="competitionRules" label-position="top">
        <el-form-item label="竞赛名称" prop="name">
          <el-input v-model="competitionForm.name" placeholder="竞赛全称" />
        </el-form-item>
        <el-button type="primary" :loading="competitionRequest.loading" @click="handleCompetitionCreate">
          新增竞赛
        </el-button>
        <el-button style="margin-left: 8px" :loading="listRequest.loading" @click="loadCompetitions">
          刷新列表
        </el-button>
      </el-form>
      <pre v-if="competitions">{{ competitions }}</pre>
    </el-card>

    <el-card class="card">
      <h3>志愿服务记录导入</h3>
      <el-form ref="volunteerImportRef" :model="volunteerImportForm" :rules="volunteerImportRules" label-position="top">
        <el-form-item label="志愿服务 Excel" prop="fileName">
          <el-upload
            :auto-upload="false"
            :limit="1"
            :show-file-list="true"
            :on-change="handleVolunteerFileChange"
          >
            <el-button>选择文件</el-button>
          </el-upload>
        </el-form-item>
        <el-button
          type="primary"
          style="margin-top: 12px"
          :loading="volunteerImportRequest.loading"
          @click="handleVolunteerImport"
        >
          导入志愿服务
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛获奖记录导入</h3>
      <el-form ref="contestImportRef" :model="contestImportForm" :rules="contestImportRules" label-position="top">
        <el-form-item label="竞赛获奖 Excel" prop="fileName">
          <el-upload
            :auto-upload="false"
            :limit="1"
            :show-file-list="true"
            :on-change="handleContestFileChange"
          >
            <el-button>选择文件</el-button>
          </el-upload>
        </el-form-item>
        <el-button
          type="primary"
          style="margin-top: 12px"
          :loading="contestImportRequest.loading"
          @click="handleContestImport"
        >
          导入竞赛获奖
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>模板配置</h3>
      <el-form ref="formFieldRef" :model="formField" :rules="formFieldRules" label-position="top">
        <el-form-item label="字段 Key" prop="field_key">
          <el-input v-model="formField.field_key" placeholder="location" />
        </el-form-item>
        <el-form-item label="字段标签" prop="label">
          <el-input v-model="formField.label" placeholder="地点" />
        </el-form-item>
        <el-form-item label="表单类型" prop="form_type">
          <el-select v-model="formField.form_type">
            <el-option label="志愿服务" value="volunteer" />
            <el-option label="竞赛获奖" value="contest" />
          </el-select>
        </el-form-item>
        <el-form-item label="字段类型" prop="field_type">
          <el-select v-model="formField.field_type">
            <el-option label="文本" value="text" />
            <el-option label="数字" value="number" />
          </el-select>
        </el-form-item>
        <el-form-item label="是否必填">
          <el-select v-model="formField.required">
            <el-option label="必填" :value="true" />
            <el-option label="可选" :value="false" />
          </el-select>
        </el-form-item>
        <el-form-item label="排序序号" prop="order_index">
          <el-input-number v-model="formField.order_index" :min="1" />
        </el-form-item>
        <el-button type="primary" :loading="formFieldRequest.loading" @click="handleFormFieldCreate">
          新增字段
        </el-button>
        <el-button style="margin-left: 8px" :loading="listRequest.loading" @click="loadFormFields">
          刷新字段
        </el-button>
      </el-form>
      <pre v-if="formFields">{{ formFields }}</pre>
    </el-card>

    <el-card class="card">
      <h3>数据删除（软删除）</h3>
      <p>仅允许删除未审核记录，删除后可在“彻底删除”页面清理。</p>
      <el-button :loading="listDataRequest.loading" @click="loadDataLists">加载列表</el-button>
      <el-button style="margin-left: 8px" @click="handleOpenPurge">进入彻底删除</el-button>

      <h4 style="margin-top: 16px">学生</h4>
      <el-table :data="students">
        <el-table-column prop="student_no" label="学号" />
        <el-table-column prop="name" label="姓名" />
        <el-table-column label="操作">
          <template #default="scope">
            <el-button
              type="danger"
              size="small"
              :loading="deleteRequest.loading"
              @click="handleDeleteStudent(scope.row.student_no)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <h4 style="margin-top: 16px">志愿服务记录</h4>
      <el-table :data="volunteerRecords">
        <el-table-column prop="title" label="标题" />
        <el-table-column prop="status" label="状态" />
        <el-table-column label="操作">
          <template #default="scope">
            <el-button
              type="danger"
              size="small"
              :disabled="scope.row.status !== 'submitted'"
              :loading="deleteRequest.loading"
              @click="handleDeleteVolunteerRecord(scope.row.id)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <h4 style="margin-top: 16px">竞赛记录</h4>
      <el-table :data="contestRecords">
        <el-table-column prop="contest_name" label="竞赛名称" />
        <el-table-column prop="status" label="状态" />
        <el-table-column label="操作">
          <template #default="scope">
            <el-button
              type="danger"
              size="small"
              :disabled="scope.row.status !== 'submitted'"
              :loading="deleteRequest.loading"
              @click="handleDeleteContestRecord(scope.row.id)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-card class="card">
      <h3>创建用户 / 发送邀请</h3>
      <el-form ref="userFormRef" :model="userForm" :rules="userRules" label-position="top">
        <el-form-item label="用户名" prop="username">
          <el-input v-model="userForm.username" placeholder="学号或工号" />
        </el-form-item>
        <el-form-item label="显示名称" prop="display_name">
          <el-input v-model="userForm.display_name" placeholder="姓名" />
        </el-form-item>
        <el-form-item label="角色" prop="role">
          <el-select v-model="userForm.role">
            <el-option label="学生" value="student" />
            <el-option label="教师" value="teacher" />
            <el-option label="审核员" value="reviewer" />
            <el-option label="管理员" value="admin" />
          </el-select>
        </el-form-item>
        <el-form-item label="邮箱（非学生必填）" prop="email">
          <el-input v-model="userForm.email" placeholder="user@example.com" />
        </el-form-item>
        <el-button type="primary" :loading="userRequest.loading" @click="handleCreateUser">
          提交
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>密码策略</h3>
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

    <el-card class="card">
      <h3>认证重置（非学生）</h3>
      <el-form :model="resetForm" label-position="top">
        <el-form-item label="用户名">
          <el-input v-model="resetForm.username" placeholder="工号" />
        </el-form-item>
        <el-form-item label="重置方式">
          <el-select v-model="resetForm.method">
            <el-option label="TOTP" value="totp" />
            <el-option label="Passkey" value="passkey" />
          </el-select>
        </el-form-item>
        <el-button type="primary" :loading="resetRequest.loading" @click="handleResetAuth">
          发送重置链接
        </el-button>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>生成一次性重置码</h3>
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
      <p style="margin-top: 8px; color: var(--muted)">重置码仅可使用一次，泄露后请重新生成。</p>
    </el-card>
  </div>

  <el-alert
    v-if="
      competitionRequest.error ||
      formFieldRequest.error ||
      importRequest.error ||
      listRequest.error ||
      competitionImportRequest.error ||
      volunteerImportRequest.error ||
      contestImportRequest.error ||
      deleteRequest.error ||
      listDataRequest.error ||
      userRequest.error ||
      policyRequest.error ||
      resetRequest.error ||
      resetCodeRequest.error
    "
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="
      competitionRequest.error ||
      formFieldRequest.error ||
      importRequest.error ||
      listRequest.error ||
      competitionImportRequest.error ||
      volunteerImportRequest.error ||
      contestImportRequest.error ||
      deleteRequest.error ||
      listDataRequest.error ||
      userRequest.error ||
      policyRequest.error ||
      resetRequest.error ||
      resetCodeRequest.error
    "
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
