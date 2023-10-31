<template>
  <div>{{ data.percent }}</div>
</template>

<script lang="ts" setup>
defineProps<{ data: { percent: number } }>();
</script>
