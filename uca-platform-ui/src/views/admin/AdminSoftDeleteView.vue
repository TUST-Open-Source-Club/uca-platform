<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { deleteContestRecord, deleteStudent, deleteVolunteerRecord } from '../../api/admin'
import { queryContest, queryVolunteer } from '../../api/records'
import { queryStudents } from '../../api/students'
import { useRequest } from '../../composables/useRequest'

const router = useRouter()
const students = ref<any[]>([])
const volunteerRecords = ref<any[]>([])
const contestRecords = ref<any[]>([])
const listDataRequest = useRequest()
const deleteRequest = useRequest()

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
</script>

<template>
  <section class="hero">
    <h1>数据删除（软删除）</h1>
    <p>仅允许删除未审核记录，删除后可在“彻底删除”页面清理。</p>
  </section>

  <div class="card-grid">
    <el-card class="card">
      <h3>软删除列表</h3>
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
  </div>

  <el-alert
    v-if="listDataRequest.error || deleteRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="listDataRequest.error || deleteRequest.error"
    :closable="false"
  />
</template>
