<script setup lang="ts">
import { onMounted, ref } from 'vue'
import {
  listDeletedContestRecords,
  listDeletedStudents,
  listDeletedVolunteerRecords,
  purgeContestRecord,
  purgeStudent,
  purgeVolunteerRecord,
} from '../api/admin'
import { useRequest } from '../composables/useRequest'

const deletedStudents = ref<any[]>([])
const deletedVolunteerRecords = ref<any[]>([])
const deletedContestRecords = ref<any[]>([])
const listRequest = useRequest()
const purgeRequest = useRequest()

const loadDeleted = async () => {
  await listRequest.run(async () => {
    const [students, volunteer, contest] = await Promise.all([
      listDeletedStudents(),
      listDeletedVolunteerRecords(),
      listDeletedContestRecords(),
    ])
    deletedStudents.value = students
    deletedVolunteerRecords.value = volunteer
    deletedContestRecords.value = contest
  })
}

const handlePurgeStudent = async (studentNo: string) => {
  await purgeRequest.run(async () => {
    await purgeStudent(studentNo)
    await loadDeleted()
  }, { successMessage: '学生已彻底删除' })
}

const handlePurgeVolunteerRecord = async (recordId: string) => {
  await purgeRequest.run(async () => {
    await purgeVolunteerRecord(recordId)
    await loadDeleted()
  }, { successMessage: '志愿记录已彻底删除' })
}

const handlePurgeContestRecord = async (recordId: string) => {
  await purgeRequest.run(async () => {
    await purgeContestRecord(recordId)
    await loadDeleted()
  }, { successMessage: '竞赛记录已彻底删除' })
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
    <el-table :data="deletedStudents">
      <el-table-column prop="student_no" label="学号" />
      <el-table-column prop="name" label="姓名" />
      <el-table-column label="操作">
        <template #default="scope">
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

    <h3 style="margin-top: 16px">已删除志愿记录</h3>
    <el-table :data="deletedVolunteerRecords">
      <el-table-column prop="title" label="标题" />
      <el-table-column prop="status" label="状态" />
      <el-table-column label="操作">
        <template #default="scope">
          <el-button
            type="danger"
            size="small"
            :loading="purgeRequest.loading"
            @click="handlePurgeVolunteerRecord(scope.row.id)"
          >
            彻底删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <h3 style="margin-top: 16px">已删除竞赛记录</h3>
    <el-table :data="deletedContestRecords">
      <el-table-column prop="contest_name" label="竞赛名称" />
      <el-table-column prop="status" label="状态" />
      <el-table-column label="操作">
        <template #default="scope">
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
      v-if="listRequest.error || purgeRequest.error"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="listRequest.error || purgeRequest.error"
      :closable="false"
    />
  </el-card>
</template>
