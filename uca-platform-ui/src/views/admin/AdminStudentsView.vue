<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { deleteStudent } from '../../api/admin'
import { queryStudents, updateStudent } from '../../api/students'
import { useRequest } from '../../composables/useRequest'

type StudentItem = {
  id: string
  student_no: string
  name: string
  gender: string
  department: string
  major: string
  class_name: string
  phone: string
}

const tableRef = ref()
const students = ref<StudentItem[]>([])
const selection = ref<StudentItem[]>([])

const filterForm = reactive({
  department: '',
  major: '',
  class_name: '',
  keyword: '',
})

const pagination = reactive({
  page: 1,
  pageSize: 10,
})

const editDialogVisible = ref(false)
const editFormRef = ref()
const editForm = reactive({
  student_no: '',
  name: '',
  gender: '',
  department: '',
  major: '',
  class_name: '',
  phone: '',
})

const listRequest = useRequest()
const saveRequest = useRequest()
const deleteRequest = useRequest()

const rules = {
  name: [{ required: true, message: '请输入姓名', trigger: 'blur' }],
  gender: [{ required: true, message: '请输入性别', trigger: 'blur' }],
  department: [{ required: true, message: '请输入院系', trigger: 'blur' }],
  major: [{ required: true, message: '请输入专业', trigger: 'blur' }],
  class_name: [{ required: true, message: '请输入班级', trigger: 'blur' }],
  phone: [{ required: true, message: '请输入手机号', trigger: 'blur' }],
}

const filteredStudents = computed(() => {
  return students.value.filter((item) => {
    if (filterForm.department && item.department !== filterForm.department) return false
    if (filterForm.major && item.major !== filterForm.major) return false
    if (filterForm.class_name && item.class_name !== filterForm.class_name) return false
    if (filterForm.keyword) {
      const keyword = filterForm.keyword.trim()
      if (keyword && !item.student_no.includes(keyword) && !item.name.includes(keyword)) {
        return false
      }
    }
    return true
  })
})

const pagedStudents = computed(() => {
  const start = (pagination.page - 1) * pagination.pageSize
  return filteredStudents.value.slice(start, start + pagination.pageSize)
})

watch(
  () => [filterForm.department, filterForm.major, filterForm.class_name, filterForm.keyword],
  () => {
    pagination.page = 1
  },
)

const loadStudents = async () => {
  await listRequest.run(async () => {
    const data = await queryStudents({
      department: filterForm.department || undefined,
      major: filterForm.major || undefined,
      class_name: filterForm.class_name || undefined,
      keyword: filterForm.keyword || undefined,
    })
    students.value = data as StudentItem[]
  })
}

const openEditDialog = (row: StudentItem) => {
  editForm.student_no = row.student_no
  editForm.name = row.name
  editForm.gender = row.gender
  editForm.department = row.department
  editForm.major = row.major
  editForm.class_name = row.class_name
  editForm.phone = row.phone
  editDialogVisible.value = true
}

const handleEditSave = async () => {
  if (!editFormRef.value) return
  await editFormRef.value.validate(async (valid: boolean) => {
    if (!valid) return
    await saveRequest.run(async () => {
      await updateStudent(editForm.student_no, {
        name: editForm.name,
        gender: editForm.gender,
        department: editForm.department,
        major: editForm.major,
        class_name: editForm.class_name,
        phone: editForm.phone,
      })
      editDialogVisible.value = false
      await loadStudents()
    }, { successMessage: '学生信息已更新' })
  })
}

const handleSelectionChange = (rows: StudentItem[]) => {
  selection.value = rows
}

const handleToggleAll = () => {
  tableRef.value?.toggleAllSelection?.()
}

const handleBulkDelete = async () => {
  if (!selection.value.length) return
  const confirmed = await ElMessageBox.confirm(
    `确认删除已选中的 ${selection.value.length} 名学生？`,
    '批量删除',
    { type: 'warning', confirmButtonText: '删除', cancelButtonText: '取消' },
  ).then(() => true).catch(() => false)
  if (!confirmed) return
  await deleteRequest.run(async () => {
    for (const item of selection.value) {
      await deleteStudent(item.student_no)
    }
    selection.value = []
    await loadStudents()
  }, { successMessage: '学生已删除' })
}

const handleSingleDelete = async (row: StudentItem) => {
  selection.value = [row]
  await handleBulkDelete()
}

onMounted(() => {
  void loadStudents()
})
</script>

<template>
  <section class="hero">
    <h1>学生名单管理</h1>
    <p>表格化维护学生信息，支持筛选、批量删除与双击编辑。</p>
  </section>

  <el-card class="card">
    <el-form label-position="top" style="display: flex; flex-wrap: wrap; gap: 12px">
      <el-form-item label="院系">
        <el-input v-model="filterForm.department" placeholder="信息学院" />
      </el-form-item>
      <el-form-item label="专业">
        <el-input v-model="filterForm.major" placeholder="软件工程" />
      </el-form-item>
      <el-form-item label="班级">
        <el-input v-model="filterForm.class_name" placeholder="软工1班" />
      </el-form-item>
      <el-form-item label="学号/姓名">
        <el-input v-model="filterForm.keyword" placeholder="关键字" />
      </el-form-item>
    </el-form>

    <div style="margin-top: 8px; display: flex; gap: 8px; justify-content: flex-end">
      <el-button :loading="listRequest.loading" @click="loadStudents">刷新列表</el-button>
      <el-button @click="handleToggleAll">全选</el-button>
      <el-button type="danger" :disabled="!selection.length" :loading="deleteRequest.loading" @click="handleBulkDelete">
        批量删除
      </el-button>
    </div>

    <el-table
      ref="tableRef"
      :data="pagedStudents"
      style="margin-top: 16px"
      @selection-change="handleSelectionChange"
      @row-dblclick="openEditDialog"
    >
      <el-table-column type="selection" width="48" />
      <el-table-column prop="student_no" label="学号" width="140" />
      <el-table-column prop="name" label="姓名" width="120" />
      <el-table-column prop="gender" label="性别" width="100" />
      <el-table-column prop="department" label="院系" min-width="160" />
      <el-table-column prop="major" label="专业" min-width="160" />
      <el-table-column prop="class_name" label="班级" width="140" />
      <el-table-column prop="phone" label="手机号" width="160" />
      <el-table-column label="操作" width="140">
        <template #default="{ row }">
          <el-button size="small" @click="openEditDialog(row)">编辑</el-button>
          <el-button size="small" type="danger" :loading="deleteRequest.loading" @click="handleSingleDelete(row)">
            删除
          </el-button>
        </template>
      </el-table-column>
    </el-table>

    <el-pagination
      v-if="filteredStudents.length"
      style="margin-top: 12px; justify-content: flex-end"
      layout="total, sizes, prev, pager, next"
      :total="filteredStudents.length"
      :page-size="pagination.pageSize"
      :current-page="pagination.page"
      @update:page-size="(size: number) => { pagination.pageSize = size; pagination.page = 1 }"
      @update:current-page="(page: number) => { pagination.page = page }"
    />

    <el-empty v-if="!filteredStudents.length" description="暂无学生数据" />
  </el-card>

  <el-dialog v-model="editDialogVisible" title="编辑学生" width="520px">
    <el-form ref="editFormRef" :model="editForm" :rules="rules" label-position="top">
      <el-form-item label="学号">
        <el-input v-model="editForm.student_no" disabled />
      </el-form-item>
      <el-form-item label="姓名" prop="name">
        <el-input v-model="editForm.name" />
      </el-form-item>
      <el-form-item label="性别" prop="gender">
        <el-input v-model="editForm.gender" />
      </el-form-item>
      <el-form-item label="院系" prop="department">
        <el-input v-model="editForm.department" />
      </el-form-item>
      <el-form-item label="专业" prop="major">
        <el-input v-model="editForm.major" />
      </el-form-item>
      <el-form-item label="班级" prop="class_name">
        <el-input v-model="editForm.class_name" />
      </el-form-item>
      <el-form-item label="手机号" prop="phone">
        <el-input v-model="editForm.phone" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="editDialogVisible = false">取消</el-button>
      <el-button type="primary" :loading="saveRequest.loading" @click="handleEditSave">保存</el-button>
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
