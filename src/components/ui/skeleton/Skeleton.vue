<!-- Skeleton.vue -->
<script setup lang="ts">
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  class?: string
}>(), {
  class: ''
})
</script>

<template>
  <div :class="cn('animate-pulse rounded-md !bg-[#27272a]/70', props.class)" />
</template>