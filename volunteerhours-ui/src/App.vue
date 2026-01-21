<script setup lang="ts"></script>

<template>
  <div class="app-shell">
    <aside class="sidebar">
      <div class="brand">VolunteerHours</div>
      <nav class="nav">
        <RouterLink to="/login">登录</RouterLink>
        <RouterLink to="/student">学生中心</RouterLink>
        <RouterLink to="/records">我的记录</RouterLink>
        <RouterLink to="/review">审核中心</RouterLink>
        <RouterLink to="/admin">管理台</RouterLink>
        <RouterLink to="/exports">导出中心</RouterLink>
        <RouterLink to="/devices">设备与恢复码</RouterLink>
      </nav>
    </aside>
    <main class="main">
      <RouterView />
    </main>
  </div>
</template>
