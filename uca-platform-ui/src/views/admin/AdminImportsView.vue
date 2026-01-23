<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import type { UploadFile } from 'element-plus'
import { importCompetitions, importContestRecords, type CompetitionSheetPlan } from '../../api/admin'
import { importStudents } from '../../api/students'
import { useRequest } from '../../composables/useRequest'

const importFormRef = ref()
const competitionImportRef = ref()
const contestImportRef = ref()
const importFile = ref<File | null>(null)
const competitionImportFile = ref<File | null>(null)
const contestImportFile = ref<File | null>(null)
const result = ref<{ key: string; value: string }[] | null>(null)
const createStudentUsers = ref(false)
const studentPasswordRule = reactive({
  prefix: 'st',
  suffix: '',
  include_student_no: true,
  include_phone: false,
})

const studentStep = ref(1)
const competitionStep = ref(1)
const contestStep = ref(1)
const competitionSheetPlan = ref<
  {
    name: string
    year: string
    name_column: string
    category_column: string
    category_suffix: 'none' | 'class' | 'class_contest'
  }[]
>([])
const studentFieldMap = ref([
  { key: 'student_no', label: '学号', column: '' },
  { key: 'name', label: '姓名', column: '' },
  { key: 'gender', label: '性别', column: '' },
  { key: 'department', label: '院系', column: '' },
  { key: 'major', label: '专业', column: '' },
  { key: 'class_name', label: '班级', column: '' },
  { key: 'phone', label: '手机号', column: '' },
])
const contestFieldMap = ref([
  { key: 'student_no', label: '学号', column: '' },
  { key: 'contest_name', label: '竞赛名称', column: '' },
  { key: 'contest_level', label: '获奖级别', column: '' },
  { key: 'contest_role', label: '角色', column: '' },
  { key: 'award_level', label: '获奖等级', column: '' },
  { key: 'self_hours', label: '自评学时', column: '' },
  { key: 'contest_year', label: '年份', column: '' },
  { key: 'contest_category', label: '竞赛类别', column: '' },
  { key: 'award_date', label: '获奖时间', column: '' },
  { key: 'first_review_hours', label: '初审学时', column: '' },
  { key: 'final_review_hours', label: '复审学时', column: '' },
  { key: 'status', label: '审核状态', column: '' },
  { key: 'rejection_reason', label: '不通过原因', column: '' },
])

const importRequest = useRequest()
const competitionImportRequest = useRequest()
const contestImportRequest = useRequest()

const importForm = reactive({
  fileName: '',
})

const competitionImportForm = reactive({
  fileName: '',
})

const contestImportForm = reactive({
  fileName: '',
})

const importRules = {
  fileName: [{ required: true, message: '请选择 Excel 文件', trigger: 'change' }],
}

const competitionImportRules = {
  fileName: [{ required: true, message: '请选择竞赛库 Excel 文件', trigger: 'change' }],
}

const contestImportRules = {
  fileName: [{ required: true, message: '请选择竞赛获奖导入文件', trigger: 'change' }],
}

const competitionMissingYear = computed(() =>
  competitionSheetPlan.value.filter((item) => item.name.trim() && !item.year.trim()),
)

const isPasswordRuleEmpty = () => {
  const prefix = studentPasswordRule.prefix?.trim() ?? ''
  const suffix = studentPasswordRule.suffix?.trim() ?? ''
  return !prefix && !suffix && !studentPasswordRule.include_student_no && !studentPasswordRule.include_phone
}

const handleImport = async () => {
  if (!importFormRef.value) return
  await importFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    if (createStudentUsers.value && isPasswordRuleEmpty()) {
      importRequest.error = '请设置密码规则，至少包含一个字段或固定字符串'
      return
    }
    await importRequest.run(
      async () => {
        const data = await importStudents(
          importFile.value as File,
          buildFieldMap(studentFieldMap.value),
          createStudentUsers.value,
          createStudentUsers.value ? { ...studentPasswordRule } : undefined,
        )
        result.value = Object.entries(data as Record<string, unknown>).map(([key, value]) => ({
          key,
          value: value === null || value === undefined ? '-' : String(value),
        }))
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

    const payload = buildCompetitionSheetPlan()
    await competitionImportRequest.run(
      async () => {
        const data = await importCompetitions(
          competitionImportFile.value as File,
          undefined,
          payload.length ? payload : undefined,
        )
        result.value = Object.entries(data as Record<string, unknown>).map(([key, value]) => ({
          key,
          value: value === null || value === undefined ? '-' : String(value),
        }))
      },
      { successMessage: '竞赛库已导入' },
    )
  })
}

const handleCompetitionFileChange = (file: UploadFile) => {
  competitionImportFile.value = file.raw ?? null
  competitionImportForm.fileName = file.name ?? ''
}

const handleContestImport = async () => {
  if (!contestImportRef.value) return
  await contestImportRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await contestImportRequest.run(
      async () => {
        const data = await importContestRecords(
          contestImportFile.value as File,
          buildFieldMap(contestFieldMap.value),
        )
        result.value = Object.entries(data as Record<string, unknown>).map(([key, value]) => ({
          key,
          value: value === null || value === undefined ? '-' : String(value),
        }))
      },
      { successMessage: '竞赛获奖记录已导入' },
    )
  })
}

const handleContestFileChange = (file: UploadFile) => {
  contestImportFile.value = file.raw ?? null
  contestImportForm.fileName = file.name ?? ''
}

const addCompetitionSheet = () => {
  competitionSheetPlan.value.push({
    name: '',
    year: '',
    name_column: '',
    category_column: '',
    category_suffix: 'none',
  })
}

const removeCompetitionSheet = (index: number) => {
  competitionSheetPlan.value.splice(index, 1)
}

const buildCompetitionSheetPlan = (): CompetitionSheetPlan[] => {
  return competitionSheetPlan.value
    .filter((item) => item.name.trim() || item.year || item.name_column || item.category_column)
    .map((item) => {
      const year = Number(item.year)
      return {
        name: item.name.trim(),
        year: Number.isFinite(year) && year > 0 ? year : null,
        name_column: item.name_column.trim() || undefined,
        category_column: item.category_column.trim() || undefined,
        category_suffix: item.category_suffix === 'none' ? undefined : item.category_suffix,
      }
    })
    .filter((item) => item.name)
}

const buildFieldMap = (rows: { key: string; column: string }[]) => {
  const map: Record<string, string> = {}
  for (const row of rows) {
    const value = row.column?.trim()
    if (value) {
      map[row.key] = value
    }
  }
  return Object.keys(map).length ? map : undefined
}

const goStudentNext = async () => {
  if (studentStep.value === 1) {
    if (!importFormRef.value) return
    await importFormRef.value.validate(async (valid: boolean) => {
      if (!valid) return
      studentStep.value = 2
    })
    return
  }
  if (studentStep.value === 2) {
    await handleImport()
  }
}

const goContestNext = async () => {
  if (contestStep.value === 1) {
    if (!contestImportRef.value) return
    await contestImportRef.value.validate(async (valid: boolean) => {
      if (!valid) return
      contestStep.value = 2
    })
    return
  }
  if (contestStep.value === 2) {
    await handleContestImport()
  }
}

const goCompetitionNext = async () => {
  if (competitionStep.value === 1) {
    if (!competitionImportRef.value) return
    await competitionImportRef.value.validate(async (valid: boolean) => {
      if (!valid) return
      competitionStep.value = 2
    })
    return
  }
  if (competitionStep.value === 2) {
    if (competitionMissingYear.value.length === 0) {
      await handleCompetitionImport()
      return
    }
    competitionStep.value = 3
    return
  }
  await handleCompetitionImport()
}

onMounted(() => {
  if (!competitionSheetPlan.value.length) {
    addCompetitionSheet()
  }
})
</script>

<template>
  <section class="hero">
    <h1>数据导入</h1>
    <p>学生名单、竞赛库与记录批量导入。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>学生名单导入</h3>
      <el-steps :active="studentStep - 1" finish-status="success" align-center style="margin: 12px 0 16px">
        <el-step title="上传文件" />
        <el-step title="字段映射" />
        <el-step title="确认导入" />
      </el-steps>
      <el-form ref="importFormRef" :model="importForm" :rules="importRules" label-position="top">
        <div v-if="studentStep === 1">
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
          <el-button type="primary" :loading="importRequest.loading" @click="goStudentNext">
            下一步
          </el-button>
        </div>
        <div v-else-if="studentStep === 2">
          <el-table :data="studentFieldMap" style="margin-bottom: 12px">
            <el-table-column label="字段">
              <template #default="{ row }">
                <span>{{ row.label }}</span>
              </template>
            </el-table-column>
            <el-table-column label="Excel 列">
              <template #default="{ row }">
                <el-input v-model="row.column" placeholder="表头/列字母/列序号" />
              </template>
            </el-table-column>
          </el-table>
          <el-switch
            v-model="createStudentUsers"
            active-text="导入同时创建学生用户"
            inactive-text="仅导入学生名单"
            style="margin-bottom: 12px"
          />
          <div v-if="createStudentUsers" style="margin-bottom: 12px">
            <el-form label-position="top" style="display: flex; flex-wrap: wrap; gap: 12px">
              <el-form-item label="密码前缀">
                <el-input v-model="studentPasswordRule.prefix" placeholder="例如 st" />
              </el-form-item>
              <el-form-item label="密码后缀">
                <el-input v-model="studentPasswordRule.suffix" placeholder="例如 @2024" />
              </el-form-item>
              <el-form-item label="包含学号">
                <el-switch v-model="studentPasswordRule.include_student_no" />
              </el-form-item>
              <el-form-item label="包含手机号">
                <el-switch v-model="studentPasswordRule.include_phone" />
              </el-form-item>
            </el-form>
          </div>
          <p style="margin-top: 8px; color: var(--muted)">
            留空表示按默认表头匹配；可填写列字母（A/B）或列序号（从 1 开始）。学生是否允许登录请到“学生名单管理”中设置。
          </p>
          <div style="display: flex; gap: 8px; margin-top: 12px">
            <el-button @click="studentStep = 1">上一步</el-button>
            <el-button type="primary" :loading="importRequest.loading" @click="goStudentNext">
              导入
            </el-button>
          </div>
        </div>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛名称库导入</h3>
      <el-steps :active="competitionStep - 1" finish-status="success" align-center style="margin: 12px 0 16px">
        <el-step title="上传文件" />
        <el-step title="映射配置" />
        <el-step title="年份设置" />
      </el-steps>
      <el-form
        ref="competitionImportRef"
        :model="competitionImportForm"
        :rules="competitionImportRules"
        label-position="top"
      >
        <div v-if="competitionStep === 1">
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
          <el-button type="primary" :loading="competitionImportRequest.loading" @click="goCompetitionNext">
            下一步
          </el-button>
        </div>
        <div v-else-if="competitionStep === 2">
          <el-table :data="competitionSheetPlan" style="margin-bottom: 12px">
            <el-table-column label="工作表名称">
              <template #default="{ row }">
                <el-input v-model="row.name" placeholder="例如 Sheet1" />
              </template>
            </el-table-column>
            <el-table-column label="年份">
              <template #default="{ row }">
                <el-input v-model="row.year" placeholder="例如 2024" />
              </template>
            </el-table-column>
            <el-table-column label="竞赛名称列">
              <template #default="{ row }">
                <el-input v-model="row.name_column" placeholder="表头/列字母/列序号" />
              </template>
            </el-table-column>
            <el-table-column label="竞赛类别列">
              <template #default="{ row }">
                <el-input v-model="row.category_column" placeholder="表头/列字母/列序号" />
              </template>
            </el-table-column>
            <el-table-column label="类别后缀" width="160">
              <template #default="{ row }">
                <el-select v-model="row.category_suffix">
                  <el-option label="无" value="none" />
                  <el-option label="类" value="class" />
                  <el-option label="类竞赛" value="class_contest" />
                </el-select>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="120">
              <template #default="{ $index }">
                <el-button size="small" type="danger" @click="removeCompetitionSheet($index)">
                  删除
                </el-button>
              </template>
            </el-table-column>
          </el-table>
          <el-button size="small" @click="addCompetitionSheet">新增工作表</el-button>
          <p style="margin-top: 8px; color: var(--muted)">
            列可填写表头名或列字母/序号。若已填写所有年份将直接导入。
          </p>
          <div style="display: flex; gap: 8px; margin-top: 12px">
            <el-button @click="competitionStep = 1">上一步</el-button>
            <el-button type="primary" :loading="competitionImportRequest.loading" @click="goCompetitionNext">
              下一步
            </el-button>
          </div>
        </div>
        <div v-else>
          <el-table :data="competitionMissingYear" style="margin-bottom: 12px">
            <el-table-column label="工作表">
              <template #default="{ row }">
                <span>{{ row.name }}</span>
              </template>
            </el-table-column>
            <el-table-column label="年份">
              <template #default="{ row }">
                <el-input v-model="row.year" placeholder="例如 2024" />
              </template>
            </el-table-column>
          </el-table>
          <p style="margin-top: 8px; color: var(--muted)">
            若工作表缺少年份列，请在此补充；填完后直接导入。
          </p>
          <div style="display: flex; gap: 8px; margin-top: 12px">
            <el-button @click="competitionStep = 2">上一步</el-button>
            <el-button type="primary" :loading="competitionImportRequest.loading" @click="goCompetitionNext">
              导入
            </el-button>
          </div>
        </div>
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛获奖记录导入</h3>
      <el-steps :active="contestStep - 1" finish-status="success" align-center style="margin: 12px 0 16px">
        <el-step title="上传文件" />
        <el-step title="字段映射" />
        <el-step title="确认导入" />
      </el-steps>
      <el-form ref="contestImportRef" :model="contestImportForm" :rules="contestImportRules" label-position="top">
        <div v-if="contestStep === 1">
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
          <el-button type="primary" :loading="contestImportRequest.loading" @click="goContestNext">
            下一步
          </el-button>
        </div>
        <div v-else-if="contestStep === 2">
          <el-table :data="contestFieldMap" style="margin-bottom: 12px">
            <el-table-column label="字段">
              <template #default="{ row }">
                <span>{{ row.label }}</span>
              </template>
            </el-table-column>
            <el-table-column label="Excel 列">
              <template #default="{ row }">
                <el-input v-model="row.column" placeholder="表头/列字母/列序号" />
              </template>
            </el-table-column>
          </el-table>
          <p style="margin-top: 8px; color: var(--muted)">
            留空表示按默认表头匹配；年份请在映射中指定年份列。
          </p>
          <div style="display: flex; gap: 8px; margin-top: 12px">
            <el-button @click="contestStep = 1">上一步</el-button>
            <el-button type="primary" :loading="contestImportRequest.loading" @click="goContestNext">
              导入
            </el-button>
          </div>
        </div>
      </el-form>
    </el-card>
  </div>

  <el-alert
    v-if="importRequest.error || competitionImportRequest.error || contestImportRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="importRequest.error || competitionImportRequest.error || contestImportRequest.error"
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <h3>导入结果</h3>
    <el-table :data="result" border>
      <el-table-column prop="key" label="字段" width="200" />
      <el-table-column prop="value" label="结果" />
    </el-table>
  </el-card>
</template>
