<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { createCompetition, deleteCompetition, listCompetitions, updateCompetition } from '../../api/admin'
import type { CompetitionItem } from '../../api/admin'
import { useRequest } from '../../composables/useRequest'

const tableRef = ref()
const competitions = ref<CompetitionItem[]>([])
const selection = ref<CompetitionItem[]>([])

const filterForm = reactive({
  name: '',
  year: '',
  category: '',
})

const pagination = reactive({
  page: 1,
  pageSize: 10,
})

const editDialogVisible = ref(false)
const createDialogVisible = ref(false)

const editFormRef = ref()
const createFormRef = ref()

const editForm = reactive({
  id: '',
  name: '',
  year: undefined as number | undefined,
  category: '',
})

const createForm = reactive({
  name: '',
  year: undefined as number | undefined,
  category: '',
})

const listRequest = useRequest()
const saveRequest = useRequest()
const deleteRequest = useRequest()

const rules = {
  name: [{ required: true, message: '请输入竞赛名称', trigger: 'blur' }],
}

const filteredCompetitions = computed(() => {
  const name = filterForm.name.trim()
  const category = filterForm.category.trim().toLowerCase()
  const year = filterForm.year.trim()
  return competitions.value.filter((item) => {
    if (name && !item.name.includes(name)) return false
    if (category && (item.category ?? '').toLowerCase() !== category) return false
    if (year && String(item.year ?? '') !== year) return false
    return true
  })
})

const pagedCompetitions = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  return filteredCompetitions.value.slice(start, start + pagination.pageSize)
})

watch(
  () => [filterForm.name, filterForm.year, filterForm.category],
  () => {
    pagination.page = 1
  },
)

const loadCompetitions = async () => {
  await listRequest.run(async () => {
    competitions.value = await listCompetitions()
  })
}

const handleCreate = async () => {
  if (!createFormRef.value) return
  await createFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await saveRequest.run(async () => {
      await createCompetition({
        name: createForm.name,
        year: createForm.year ?? null,
        category: createForm.category || null,
      })
      createDialogVisible.value = false
      createForm.name = ''
      createForm.year = undefined
      createForm.category = ''
      await loadCompetitions()
    }, { successMessage: '已新增竞赛' })
  })
}

const handleEdit = async () => {
  if (!editFormRef.value) return
  await editFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await saveRequest.run(async () => {
      await updateCompetition(editForm.id, {
        name: editForm.name,
        year: editForm.year ?? null,
        category: editForm.category || null,
      })
      editDialogVisible.value = false
      await loadCompetitions()
    }, { successMessage: '竞赛已更新' })
  })
}

const openEditDialog = (row: CompetitionItem) => {
  editForm.id = row.id
  editForm.name = row.name
  editForm.year = row.year ?? undefined
  editForm.category = row.category ?? ''
  editDialogVisible.value = true
}

const handleSelectionChange = (rows: CompetitionItem[]) => {
  selection.value = rows
}

const handleBulkDelete = async () => {
  if (!selection.value.length) return
  const confirmed = await ElMessageBox.confirm(
    `确认删除已选中的 ${selection.value.length} 条竞赛记录？`,
    '批量删除',
    { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' },
  ).then(() => true).catch(() => false)
  if (!confirmed) return
  await deleteRequest.run(async () => {
    for (const item of selection.value) {
      await deleteCompetition(item.id)
    }
    selection.value = []
    await loadCompetitions()
  }, { successMessage: '已删除竞赛' })
}

const handleToggleAll = () => {
  tableRef.value?.toggleAllSelection?.()
}

onMounted(() => {
  void loadCompetitions()
})
</script>

<template>
  <section class="hero">
    <h1>竞赛库管理</h1>
    <p>表格化维护竞赛名称库，支持筛选、批量删除与双击编辑。</p>
  </section>

  <el-card class="card">
    <div style="display: flex; flex-wrap: wrap; gap: 12px; align-items: flex-end">
      <el-form label-position="top" style="display: flex; flex-wrap: wrap; gap: 12px">
        <el-form-item label="竞赛名称">
          <el-input v-model="filterForm.name" placeholder="关键字" />
        </el-form-item>
        <el-form-item label="竞赛年份">
          <el-input v-model="filterForm.year" placeholder="2024" />
        </el-form-item>
        <el-form-item label="竞赛类型">
          <el-select v-model="filterForm.category" clearable placeholder="A/B">
            <el-option label="A 类" value="A" />
            <el-option label="B 类" value="B" />
          </el-select>
        </el-form-item>
      </el-form>
      <div style="margin-left: auto; display: flex; gap: 8px">
        <el-button :loading="listRequest.loading" @click="loadCompetitions">刷新列表</el-button>
        <el-button @click="handleToggleAll">全选</el-button>
        <el-button type="danger" :disabled="!selection.length" :loading="deleteRequest.loading" @click="handleBulkDelete">
          批量删除
        </el-button>
        <el-button type="primary" @click="createDialogVisible = true">新增竞赛</el-button>
      </div>
    </div>

    <el-table
      ref="tableRef"
      :data="pagedCompetitions"
      style="margin-top: 16px"
      @selection-change="handleSelectionChange"
      @row-dblclick="openEditDialog"
    >
      <el-table-column type="selection" width="48" />
      <el-table-column prop="name" label="竞赛名称" min-width="220" />
      <el-table-column prop="year" label="年份" width="120" />
      <el-table-column prop="category" label="类型" width="120" />
      <el-table-column label="操作" width="140">
        <template #default="{ row }">
          <el-button size="small" @click="openEditDialog(row)">编辑</el-button>
          <el-button
            size="small"
            type="danger"
            :loading="deleteRequest.loading"
            @click="selection = [row]; handleBulkDelete()"
          >
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-if="filteredCompetitions.length"
      style="margin-top: 12px; justify-content: flex-end"
      layout="total, sizes, prev, pager, next"
      :total="filteredCompetitions.length"
      :page-size="pagination.pageSize"
      :current-page="pagination.page"
      @update:page-size="(size: number) => { pagination.pageSize = size; pagination.page = 1 }"
      @update:current-page="(page: number) => { pagination.page = page }"
    />

    <el-empty v-if="!filteredCompetitions.length" description="暂无竞赛数据" />
  </el-card>

  <el-dialog v-model="createDialogVisible" title="新增竞赛" width="480px">
    <el-form ref="createFormRef" :model="createForm" :rules="rules" label-position="top">
      <el-form-item label="竞赛名称" prop="name">
        <el-input v-model="createForm.name" placeholder="竞赛全称" />
      </el-form-item>
      <el-form-item label="竞赛年份">
        <el-input-number v-model="createForm.year" :min="2000" :max="2100" />
      </el-form-item>
      <el-form-item label="竞赛类型">
        <el-select v-model="createForm.category" placeholder="A/B">
          <el-option label="A 类" value="A" />
          <el-option label="B 类" value="B" />
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="createDialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="saveRequest.loading" @click="handleCreate">保存</el-button>
    </template>
  </el-dialog>

  <el-dialog v-model="editDialogVisible" title="编辑竞赛" width="480px">
    <el-form ref="editFormRef" :model="editForm" :rules="rules" label-position="top">
      <el-form-item label="竞赛名称" prop="name">
        <el-input v-model="editForm.name" placeholder="竞赛全称" />
      </el-form-item>
      <el-form-item label="竞赛年份">
        <el-input-number v-model="editForm.year" :min="2000" :max="2100" />
      </el-form-item>
      <el-form-item label="竞赛类型">
        <el-select v-model="editForm.category" placeholder="A/B">
          <el-option label="A 类" value="A" />
          <el-option label="B 类" value="B" />
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="editDialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="saveRequest.loading" @click="handleEdit">保存</el-button>
    </template>
  </el-dialog>

  <el-alert
    v-if="listRequest.error || saveRequest.error || deleteRequest.error"
    class="card"
    style="margin-top: 24px"
    type="error"
    show-icon
    :title="listRequest.error || saveRequest.error || deleteRequest.error"
    :closable="false"
  />
</template>
