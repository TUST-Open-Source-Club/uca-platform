<script setup lang="ts">
import { ref } from 'vue'
import { queryContest, queryVolunteer } from '../api/records'
import { useRequest } from '../composables/useRequest'

const status = ref('')
const volunteer = ref('')
const contest = ref('')
const request = useRequest()

const handleLoad = async () => {
  await request.run(
    async () => {
      const [volunteerData, contestData] = await Promise.all([
        queryVolunteer(status.value || undefined),
        queryContest(status.value || undefined),
      ])
      volunteer.value = JSON.stringify(volunteerData, null, 2)
      contest.value = JSON.stringify(contestData, null, 2)
    },
    { successMessage: '已加载记录' },
  )
}
</script>

<template>
  <section class="hero">
    <h1>我的记录</h1>
    <p>查看志愿服务与竞赛获奖审核进度与附件。</p>
  </section>

  <el-card class="card">
    <el-form label-position="top">
      <el-form-item label="状态筛选">
        <el-select v-model="status" placeholder="请选择状态" clearable>
          <el-option label="全部" value="" />
          <el-option label="已提交" value="submitted" />
          <el-option label="已初审" value="first_reviewed" />
          <el-option label="已复审" value="final_reviewed" />
          <el-option label="不通过" value="rejected" />
        </el-select>
      </el-form-item>
      <el-button type="primary" :loading="request.loading" @click="handleLoad">加载记录</el-button>
    </el-form>
  </el-card>

  <el-alert
    v-if="request.error"
    class="card"
    style="margin-top: 16px"
    type="error"
    show-icon
    :title="request.error"
    :closable="false"
  />

  <div class="card-grid" style="margin-top: 20px">
    <el-card class="card">
      <h3>志愿服务记录</h3>
      <pre v-if="volunteer">{{ volunteer }}</pre>
    </el-card>
    <el-card class="card">
      <h3>竞赛获奖记录</h3>
      <pre v-if="contest">{{ contest }}</pre>
    </el-card>
  </div>
</template>
