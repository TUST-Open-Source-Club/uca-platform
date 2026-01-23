<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { listCompetitionsPublic, type CompetitionItem } from '../api/catalog'
import { useRequest } from '../composables/useRequest'

const competitions = ref<CompetitionItem[]>([])
const request = useRequest()
const filterForm = reactive({
  year: '',
})

const loadCompetitions = async () => {
  await request.run(
    async () => {
      competitions.value = await listCompetitionsPublic()
    },
    { silent: true },
  )
}

onMounted(() => {
  void loadCompetitions()
})

const filteredCompetitions = computed(() => {
  const year = filterForm.year.trim()
  return competitions.value.filter((item) => {
    if (year && String(item.year ?? '') !== year) return false
    return true
  })
})
</script>

<template>
  <section class="hero">
    <h1>竞赛清单</h1>
    <p>本页面无需登录，仅提供竞赛名称查询。</p>
  </section>

  <el-card class="card">
    <el-form label-position="top" style="display: flex; flex-wrap: wrap; gap: 12px">
      <el-form-item label="年份">
        <el-input v-model="filterForm.year" placeholder="2024" />
      </el-form-item>
    </el-form>
    <el-table v-if="filteredCompetitions.length" :data="filteredCompetitions">
      <el-table-column prop="year" label="年份" width="120" />
      <el-table-column prop="category" label="类型" width="100" />
      <el-table-column prop="name" label="竞赛名称" />
    </el-table>
    <el-empty v-else :description="request.loading ? '加载中' : '暂无竞赛数据'" />
    <el-alert
      v-if="request.error"
      style="margin-top: 12px"
      type="error"
      show-icon
      :title="request.error"
      :closable="false"
    />
  </el-card>
</template>
