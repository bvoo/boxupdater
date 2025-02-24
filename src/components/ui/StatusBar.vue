<script setup lang="ts">
import { computed } from 'vue'
import { Button } from './button'
import { Progress } from './progress'

const steps = [
  { id: 'waiting', label: 'Waiting for BOOTSEL' },
  { id: 'ready', label: 'Ready to flash' },
  { id: 'downloading', label: 'Downloading Firmware' },
  { id: 'nuking', label: 'Nuking device' },
  { id: 'done', label: 'Done!' }
]

const props = defineProps<{
  currentStatus: string
  downloadProgress?: number
  loading?: boolean
  canFlash?: boolean
  isFlashing?: boolean
}>()

const isWaitingForBootsel = computed(() => {
  return props.currentStatus.toLowerCase().includes('waiting for bootsel')
})

defineEmits<{
  (e: 'flash'): void
}>()

const currentStepIndex = computed(() => {
  return steps.findIndex(step => 
    props.currentStatus.toLowerCase().includes(step.id.toLowerCase()) ||
    props.currentStatus.toLowerCase().includes(step.label.toLowerCase())
  )
})
</script>

<template>
  <div class="fixed bottom-0 left-0 right-0 bg-[#0C0C0D] border-t border-[#3f3f46] p-4">
    <div class="max-w-4xl mx-auto flex items-center justify-between gap-4">
      <!-- Status text on the left -->
      <div class="text-sm whitespace-nowrap">{{ currentStatus || 'Ready' }}</div>

      <!-- Progress bar in the middle -->
      <div class="flex-1 flex items-center gap-2">
        <Progress 
          class="w-full" 
          :model-value="isFlashing ? (downloadProgress || Math.max((currentStepIndex + 1) / steps.length * 100, 0)) : 0" 
        />
      </div>

      <!-- Flash button on the right -->
      <Button 
        @click="$emit('flash')" 
        :disabled="loading || !canFlash || isWaitingForBootsel"
        size="sm"
      >
        {{ loading ? 'Processing...' : 'Flash Device' }}
      </Button>
    </div>
  </div>
</template>