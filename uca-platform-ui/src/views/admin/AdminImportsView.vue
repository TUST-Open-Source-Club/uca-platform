<script setup lang="ts">
import { reactive, ref } from 'vue'
import type { UploadFile } from 'element-plus'
import {
  importCompetitions,
  importContestRecords,
  importVolunteerRecords,
} from '../../api/admin'
import { importStudents } from '../../api/students'
import { useRequest } from '../../composables/useRequest'

const importFormRef = ref()
const competitionImportRef = ref()
const volunteerImportRef = ref()
const contestImportRef = ref()
const importFile = ref<File | null>(null)
const competitionImportFile = ref<File | null>(null)
const volunteerImportFile = ref<File | null>(null)
const contestImportFile = ref<File | null>(null)
const result = ref('')

const importRequest = useRequest()
const competitionImportRequest = useRequest()
const volunteerImportRequest = useRequest()
const contestImportRequest = useRequest()

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
  </div>

  <el-alert
    v-if="
      importRequest.error ||
      competitionImportRequest.error ||
      volunteerImportRequest.error ||
      contestImportRequest.error
    "
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="
      importRequest.error ||
      competitionImportRequest.error ||
      volunteerImportRequest.error ||
      contestImportRequest.error
    "
    :closable="false"
  />
  <el-card v-if="result" class="card" style="margin-top: 24px">
    <pre>{{ result }}</pre>
  </el-card>
</template>
