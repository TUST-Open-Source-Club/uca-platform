<script setup lang="ts"></script>

<template>
  <section class="hero">
    <h1>学生中心</h1>
    <p>提交志愿服务与竞赛获奖记录，跟踪审核进度。</p>
  </section>

  <div class="card-grid">
    <article class="card">
      <h3>志愿服务填报</h3>
      <div class="form-field">
        <label>标题</label>
        <input placeholder="活动名称" />
      </div>
      <div class="form-field">
        <label>自评学时</label>
        <input type="number" placeholder="例如 4" />
      </div>
      <div class="form-field">
        <label>说明</label>
        <textarea rows="3" placeholder="简要描述服务内容"></textarea>
      </div>
      <button class="button">提交</button>
    </article>

    <article class="card">
      <h3>竞赛获奖填报</h3>
      <div class="form-field">
        <label>竞赛名称</label>
        <input placeholder="竞赛全称" />
      </div>
      <div class="form-field">
        <label>获奖等级</label>
        <input placeholder="例如 省赛一等奖" />
      </div>
      <div class="form-field">
        <label>自评学时</label>
        <input type="number" placeholder="例如 8" />
      </div>
      <button class="button">提交</button>
    </article>
  </div>
</template>
