<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import type { UploadFile } from 'element-plus'
import {
  importCompetitions,
  importContestRecords,
  listImportTemplates,
  type ImportTemplate,
} from '../../api/admin'
import { importStudents } from '../../api/students'
import { useRequest } from '../../composables/useRequest'

const importFormRef = ref()
const competitionImportRef = ref()
const contestImportRef = ref()
const importFile = ref<File | null>(null)
const competitionImportFile = ref<File | null>(null)
const contestImportFile = ref<File | null>(null)
const competitionDefaultYear = ref<number | null>(null)
const importTemplates = ref<ImportTemplate[]>([])
const showYearDialog = ref(false)
const result = ref('')

const importRequest = useRequest()
const competitionImportRequest = useRequest()
const contestImportRequest = useRequest()
const templateRequest = useRequest()

const importForm = reactive({
  fileName: '',
})

const competitionImportForm = reactive({
  fileName: '',
})

const contestImportForm = reactive({
  fileName: '',
})

const yearDialogForm = reactive({
  year: '',
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

const loadImportTemplates = async () => {
  await templateRequest.run(async () => {
    importTemplates.value = await listImportTemplates()
  })
}

const needsDefaultYear = () => {
  const template = importTemplates.value.find((item) => item.template_key === 'competition_library')
  const yearField = template?.fields.find((field) => field.field_key === 'contest_year')
  return !yearField || !yearField.column_title || yearField.column_title.trim() === ''
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
    if (needsDefaultYear() && !competitionDefaultYear.value) {
      showYearDialog.value = true
      return
    }
    try {
      await competitionImportRequest.run(async () => {
        const data = await importCompetitions(
          competitionImportFile.value as File,
          competitionDefaultYear.value,
        )
        result.value = JSON.stringify(data, null, 2)
      }, { successMessage: '竞赛库已导入' })
    } catch {
      if (competitionImportRequest.error.includes('default_year required')) {
        showYearDialog.value = true
      }
    }
  })
}

const handleCompetitionFileChange = (file: UploadFile) => {
  competitionImportFile.value = file.raw ?? null
  competitionImportForm.fileName = file.name ?? ''
}

const handleYearDialogConfirm = async () => {
  const parsed = Number(yearDialogForm.year)
  if (!Number.isFinite(parsed) || parsed <= 0) {
    competitionImportRequest.error = '请输入有效年份'
    return
  }
  competitionDefaultYear.value = parsed
  showYearDialog.value = false
  await handleCompetitionImport()
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

onMounted(() => {
  void loadImportTemplates()
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
      <el-alert
        v-if="needsDefaultYear()"
        style="margin-top: 12px"
        type="warning"
        show-icon
        title="当前导入模板未映射年份列，请在导入时设置默认年份。"
        :closable="false"
      />
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
  </div>

  <el-dialog v-model="showYearDialog" title="设置默认年份" width="420px">
    <el-form label-position="top">
      <el-form-item label="默认年份">
        <el-input v-model="yearDialogForm.year" placeholder="例如 2024" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="showYearDialog = false">取消</el-button>
      <el-button type="primary" @click="handleYearDialogConfirm">确认</el-button>
    </template>
  </el-dialog>

  <el-alert
    v-if="
      importRequest.error ||
      competitionImportRequest.error ||
      contestImportRequest.error ||
      templateRequest.error
    "
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="
      importRequest.error ||
      competitionImportRequest.error ||
      contestImportRequest.error ||
      templateRequest.error
    "
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
