<script setup lang="ts"></script>

<template>
  <section class="hero">
    <h1>导出中心</h1>
    <p>按学院、专业或个人导出 Excel/PDF。</p>
  </section>

  <div class="card-grid">
    <article class="card">
      <h3>学院汇总表</h3>
      <p>全学院学时汇总。</p>
      <button class="button">导出 Excel</button>
    </article>
    <article class="card">
      <h3>专业汇总表</h3>
      <p>按专业筛选导出。</p>
      <button class="button secondary">筛选导出</button>
    </article>
    <article class="card">
      <h3>个人专项表</h3>
      <p>单人学时专项报表。</p>
      <button class="button">导出个人表</button>
    </article>
  </div>
</template>
