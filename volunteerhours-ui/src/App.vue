<script setup lang="ts"></script>

<template>
  <el-container class="app-shell">
    <el-aside class="sidebar" width="260px">
      <div class="brand">VolunteerHours</div>
      <el-menu router class="nav" background-color="transparent" text-color="#1f4d63">
        <el-menu-item index="/login">登录</el-menu-item>
        <el-menu-item index="/student">学生中心</el-menu-item>
        <el-menu-item index="/records">我的记录</el-menu-item>
        <el-menu-item index="/review">审核中心</el-menu-item>
        <el-menu-item index="/admin">管理台</el-menu-item>
        <el-menu-item index="/exports">导出中心</el-menu-item>
        <el-menu-item index="/devices">设备与恢复码</el-menu-item>
      </el-menu>
    </el-aside>
    <el-main class="main">
      <RouterView />
    </el-main>
  </el-container>
</template>
