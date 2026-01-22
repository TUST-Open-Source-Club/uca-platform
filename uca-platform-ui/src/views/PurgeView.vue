<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
  listDeletedContestRecords,
  listDeletedStudents,
  purgeContestRecord,
  purgeStudent,
  restoreContestRecord,
  restoreStudent,
} from '../api/admin'
import { useRequest } from '../composables/useRequest'

const deletedStudents = ref<any[]>([])
const deletedContestRecords = ref<any[]>([])
const selectedStudents = ref<any[]>([])
const selectedContestRecords = ref<any[]>([])
const listRequest = useRequest()
const purgeRequest = useRequest()
const restoreRequest = useRequest()

const loadDeleted = async () => {
  await listRequest.run(async () => {
    const [students, contest] = await Promise.all([
      listDeletedStudents(),
      listDeletedContestRecords(),
    ])
    deletedStudents.value = students
    deletedContestRecords.value = contest
  })
}

const handlePurgeStudent = async (studentNo: string) => {
  await purgeRequest.run(async () => {
    await purgeStudent(studentNo)
    await loadDeleted()
  }, { successMessage: '学生已彻底删除' })
}

const handlePurgeContestRecord = async (recordId: string) => {
  await purgeRequest.run(async () => {
    await purgeContestRecord(recordId)
    await loadDeleted()
  }, { successMessage: '竞赛记录已彻底删除' })
}

const handleRestoreStudent = async (studentNo: string) => {
  await restoreRequest.run(async () => {
    await restoreStudent(studentNo)
    await loadDeleted()
  }, { successMessage: '学生已恢复' })
}

const handleRestoreContestRecord = async (recordId: string) => {
  await restoreRequest.run(async () => {
    await restoreContestRecord(recordId)
    await loadDeleted()
  }, { successMessage: '竞赛记录已恢复' })
}

const handleBulkPurgeStudents = async () => {
  const items = selectedStudents.value.map((row) => row.student_no)
  if (!items.length) return
  await purgeRequest.run(async () => {
    await Promise.all(items.map((studentNo) => purgeStudent(studentNo)))
    await loadDeleted()
  }, { successMessage: '已批量彻底删除学生' })
}

const handleBulkPurgeContestRecords = async () => {
  const items = selectedContestRecords.value.map((row) => row.id)
  if (!items.length) return
  await purgeRequest.run(async () => {
    await Promise.all(items.map((recordId) => purgeContestRecord(recordId)))
    await loadDeleted()
  }, { successMessage: '已批量彻底删除竞赛记录' })
}

const handleBulkRestoreStudents = async () => {
  const items = selectedStudents.value.map((row) => row.student_no)
  if (!items.length) return
  await restoreRequest.run(async () => {
    await Promise.all(items.map((studentNo) => restoreStudent(studentNo)))
    await loadDeleted()
  }, { successMessage: '已批量恢复学生' })
}

const handleBulkRestoreContestRecords = async () => {
  const items = selectedContestRecords.value.map((row) => row.id)
  if (!items.length) return
  await restoreRequest.run(async () => {
    await Promise.all(items.map((recordId) => restoreContestRecord(recordId)))
    await loadDeleted()
  }, { successMessage: '已批量恢复竞赛记录' })
}

const handleStudentSelection = (rows: any[]) => {
  selectedStudents.value = rows
}

const handleContestSelection = (rows: any[]) => {
  selectedContestRecords.value = rows
}

onMounted(() => {
  void loadDeleted()
})
</script>

<template>
  <section class="hero">
    <h1>彻底删除</h1>
    <p>仅管理员可操作。此操作不可恢复，请谨慎执行。</p>
  </section>

  <el-card class="card">
    <el-button :loading="listRequest.loading" @click="loadDeleted">加载已删除数据</el-button>

    <h3 style="margin-top: 16px">已删除学生</h3>
    <div style="display: flex; gap: 8px; margin-bottom: 8px">
      <el-button :loading="restoreRequest.loading" @click="handleBulkRestoreStudents">
        批量恢复
      </el-button>
      <el-button type="danger" :loading="purgeRequest.loading" @click="handleBulkPurgeStudents">
        批量彻底删除
      </el-button>
    </div>
    <el-table :data="deletedStudents" @selection-change="handleStudentSelection">
      <el-table-column type="selection" width="48" />
      <el-table-column prop="student_no" label="学号" />
      <el-table-column prop="name" label="姓名" />
      <el-table-column label="操作">
        <template #default="scope">
          <el-button
            size="small"
            :loading="restoreRequest.loading"
            @click="handleRestoreStudent(scope.row.student_no)"
          >
            恢复
          </el-button>
          <el-button
            type="danger"
            size="small"
            :loading="purgeRequest.loading"
            @click="handlePurgeStudent(scope.row.student_no)"
          >
            彻底删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <h3 style="margin-top: 16px">已删除竞赛记录</h3>
    <div style="display: flex; gap: 8px; margin-bottom: 8px">
      <el-button :loading="restoreRequest.loading" @click="handleBulkRestoreContestRecords">
        批量恢复
      </el-button>
      <el-button type="danger" :loading="purgeRequest.loading" @click="handleBulkPurgeContestRecords">
        批量彻底删除
      </el-button>
    </div>
    <el-table :data="deletedContestRecords" @selection-change="handleContestSelection">
      <el-table-column type="selection" width="48" />
      <el-table-column prop="contest_name" label="竞赛名称" />
      <el-table-column prop="status" label="状态" />
      <el-table-column label="操作">
        <template #default="scope">
          <el-button
            size="small"
            :loading="restoreRequest.loading"
            @click="handleRestoreContestRecord(scope.row.id)"
          >
            恢复
          </el-button>
          <el-button
            type="danger"
            size="small"
            :loading="purgeRequest.loading"
            @click="handlePurgeContestRecord(scope.row.id)"
          >
            彻底删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-alert
      v-if="listRequest.error || purgeRequest.error || restoreRequest.error"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="listRequest.error || purgeRequest.error || restoreRequest.error"
      :closable="false"
    />
  </el-card>
</template>
