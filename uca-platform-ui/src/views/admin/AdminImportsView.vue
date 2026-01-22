<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
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
const result = ref('')

const allowStudentLogin = ref(false)
const showStudentDialog = ref(false)
const showCompetitionDialog = ref(false)
const showContestDialog = ref(false)
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
  { key: 'contest_level', label: '竞赛级别', column: '' },
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

const handleImport = async () => {
  if (!importFormRef.value) return
  await importFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await importRequest.run(
      async () => {
        const data = await importStudents(
          importFile.value as File,
          buildFieldMap(studentFieldMap.value),
          allowStudentLogin.value,
        )
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

    const payload = buildCompetitionSheetPlan()
    await competitionImportRequest.run(
      async () => {
        const data = await importCompetitions(
          competitionImportFile.value as File,
          undefined,
          payload.length ? payload : undefined,
        )
        result.value = JSON.stringify(data, null, 2)
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
        result.value = JSON.stringify(data, null, 2)
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
        <div style="display: flex; gap: 8px; margin-top: 8px">
          <el-button @click="showStudentDialog = true">配置导入</el-button>
          <el-button type="primary" :loading="importRequest.loading" @click="handleImport">
            上传名单
          </el-button>
        </div>
        <el-switch
          v-model="allowStudentLogin"
          active-text="允许学生登录"
          inactive-text="禁止学生登录"
          style="margin-top: 12px"
        />
      </el-form>
    </el-card>

    <el-card class="card">
      <h3>竞赛名称库导入</h3>
      <el-form
        ref="competitionImportRef"
        :model="competitionImportForm"
        :rules="competitionImportRules"
        label-position="top"
      >
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
        <div style="display: flex; gap: 8px; margin-top: 8px">
          <el-button @click="showCompetitionDialog = true">配置导入</el-button>
          <el-button
            type="primary"
            :loading="competitionImportRequest.loading"
            @click="handleCompetitionImport"
          >
            导入竞赛库
          </el-button>
        </div>
        <p style="margin-top: 8px; color: var(--muted)">
          通过导入弹窗选择工作表、年份与列映射；列可填写表头名或列字母/序号。
        </p>
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
        <div style="display: flex; gap: 8px; margin-top: 8px">
          <el-button @click="showContestDialog = true">配置导入</el-button>
          <el-button type="primary" :loading="contestImportRequest.loading" @click="handleContestImport">
            导入竞赛获奖
          </el-button>
        </div>
      </el-form>
    </el-card>
  </div>

  <el-dialog v-model="showStudentDialog" title="学生名单导入配置" width="640px">
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
    <p style="margin-top: 8px; color: var(--muted)">
      留空表示按默认表头匹配；可填写列字母（A/B）或列序号（从 1 开始）。
    </p>
    <template #footer>
      <el-button @click="showStudentDialog = false">关闭</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="showCompetitionDialog" title="竞赛库导入配置" width="860px">
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
    <p style="margin-top: 12px; color: var(--muted)">
      未填写年份时将尝试读取“年份/年度”列；若缺失会提示补充年份。
    </p>
    <template #footer>
      <el-button @click="showCompetitionDialog = false">关闭</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="showContestDialog" title="竞赛获奖导入配置" width="720px">
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
      留空表示按默认表头匹配；可填写列字母（A/B）或列序号（从 1 开始）。
    </p>
    <template #footer>
      <el-button @click="showContestDialog = false">关闭</el-button>
    </template>
  </el-dialog>

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
    <pre>{{ result }}</pre>
  </el-card>
</template>
