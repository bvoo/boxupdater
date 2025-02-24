<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Button } from './components/ui/button'
import { 
  Table,
  TableBody,
  TableCell,
  TableRow 
} from './components/ui/table'
import { Tabs, TabsList, TabsTrigger } from './components/ui/tabs'
import { fetchReleasesByRepo, type Release } from './lib/repositories'
import StatusBar from './components/ui/StatusBar.vue'
import Skeleton from './components/ui/skeleton/Skeleton.vue'
import { XIcon, PlusIcon } from 'lucide-vue-next'
import {
  PopoverRoot,
  PopoverContent,
  PopoverTrigger,
} from 'reka-ui'
import { useRepositoryStore } from './stores/repository'

const store = useRepositoryStore()
const repositories = computed(() => store.repositories)

const selectedRepoId = ref<string>('')
const releases = ref<Release[]>([])
const selectedVersion = ref<string>('')
const status = ref('Waiting for BOOTSEL')
const loading = ref(false)
const selectedFileUrl = ref<string>('')
const downloadProgress = ref(0)
const isFlashing = ref(false)
const nukeBeforeFlash = ref(true)

const showAddRepo = ref(false)
const newRepoUrl = ref('')
const repoUrlError = ref('')

const selectedRepo = computed(() => {
  return repositories.value.find(r => r.name === selectedRepoId.value)
})

const firmwareVersions = computed(() => {
  const versions = new Set<string>()
  for (const release of releases.value) {
    versions.add(release.tag_name)
  }
  return Array.from(versions).sort((a, b) => {
    const versionA = a.replace(/^v/, '')
    const versionB = b.replace(/^v/, '')
    return versionB.localeCompare(versionA, undefined, { numeric: true })
  })
})

const selectedReleases = computed(() => {
  if (!selectedVersion.value) return []
  return releases.value.filter(r => r.tag_name === selectedVersion.value)
})

async function loadRepositories() {
  try {
    if (store.repositories.length > 0) {
      selectedRepoId.value = store.repositories[0].name
      await fetchReleases() // Fetch releases for the initial repository
    }
  } catch (error) {
    status.value = 'Error loading repositories'
    console.error(error)
  }
}

async function fetchReleases() {
  if (!selectedRepo.value) return
  
  try {
    loading.value = true
    status.value = 'Fetching releases...'
    releases.value = await fetchReleasesByRepo(selectedRepo.value.name)
    
    // Set the first version as default if there are any
    if (firmwareVersions.value.length > 0) {
      selectedVersion.value = firmwareVersions.value[0]
      // Removed auto-selection of first file URL
    }
    
    status.value = ''
  } catch (error) {
    status.value = 'Error fetching releases'
    console.error(error)
  } finally {
    loading.value = false
  }
}

async function flashDevice() {
  if (!selectedFileUrl.value) {
    status.value = 'Please select a file first'
    return
  }

  try {
    loading.value = true
    isFlashing.value = true
    
    status.value = 'Waiting for BOOTSEL'
    downloadProgress.value = 0
    status.value = 'Downloading firmware'
    downloadProgress.value = 33  // Start at 33%
    const fileData = await invoke('download_firmware', { url: selectedFileUrl.value })
    // No need to set to 66 here as the events will handle it
    
    if (nukeBeforeFlash.value) {
      status.value = 'Nuking device'
      await invoke('write_to_rp2', { isNuke: true })
      downloadProgress.value = 85
    }
    
    status.value = 'Writing firmware'
    await invoke('write_to_rp2', { 
      isNuke: false,
      fileData
    })
    
    downloadProgress.value = 100
    status.value = 'Done!'
  } catch (error) {
    status.value = `Error: ${error}`
    console.error(error)
  } finally {
    loading.value = false
    setTimeout(() => {
      downloadProgress.value = 0
      isFlashing.value = false
      if (!status.value.includes('Error')) {
        status.value = 'Ready to flash'
      }
    }, 1000)
  }
}

async function addCustomRepository() {
  try {
    repoUrlError.value = ''
    const parsed = parseGithubUrl(newRepoUrl.value)
    if (!parsed) {
      repoUrlError.value = 'Invalid GitHub URL'
      return
    }

    store.addRepository({
      owner: parsed.owner,
      name: parsed.name,
      description: parsed.name,
      asset_filter: "\\.uf2$",
    })

    await loadRepositories()
    showAddRepo.value = false
    newRepoUrl.value = ''
  } catch (error) {
    status.value = 'Error adding repository'
    console.error(error)
  }
}

async function removeRepository(name: string) {
  try {
    store.removeRepository(name)
    if (selectedRepoId.value === name) {
      selectedRepoId.value = repositories.value[0]?.name || ''
    }
    await loadRepositories()
  } catch (error) {
    status.value = 'Error removing repository'
    console.error(error)
  }
}

function parseGithubUrl(url: string): { owner: string, name: string } | null {
  try {
    const urlObj = new URL(url)
    if (urlObj.hostname !== 'github.com') return null
    
    const parts = urlObj.pathname.split('/').filter(Boolean)
    if (parts.length < 2) return null
    
    return {
      owner: parts[0],
      name: parts[1]
    }
  } catch {
    return null
  }
}

watch(selectedRepo, () => {
  selectedVersion.value = ''
  fetchReleases()
})

watch(selectedVersion, () => {
  selectedFileUrl.value = ''
})

onMounted(async () => {
  loadRepositories()
  
  // Listen for download progress updates, scaling from 33-66%
  await listen('download-progress', (event) => {
    const rawProgress = event.payload as DownloadProgress
    downloadProgress.value = 33 + (rawProgress.progress * 0.33) // Scale progress to range 33-66
  })

  // Set up periodic drive check
  setInterval(async () => {
    if (!loading.value && !status.value.includes('Error')) {
      try {
        const driveExists = await invoke('check_rp2_drive')
        if (!driveExists) {
          status.value = 'Waiting for BOOTSEL'
        } else {
          status.value = 'Ready to flash'
        }
      } catch (error) {
        console.error(error)
      }
    }
  }, 500)
})

interface DownloadProgress {
  progress: number
}
</script>

<template>
  <div class="grid grid-rows-[auto_1fr_auto] h-screen bg-background text-foreground overflow-hidden">
    <!-- Header -->
    <div class="max-w-4xl w-full mx-auto px-5 py-2">
      <h1 class="scroll-m-20 text-4xl font-extrabold tracking-tight lg:text-5xl">Box Updater</h1>
    </div>

    <!-- Main content -->
    <div class="max-w-4xl w-full mx-auto px-5 flex flex-col overflow-hidden">
      <!-- Repository tabs -->
      <div class="py-2">
        <div class="flex justify-center items-center gap-4">
          <Tabs v-model="selectedRepoId" class="w-fit">
            <TabsList class="flex overflow-x-auto hide-scrollbar">
              <template v-if="loading && repositories.length === 0">
                <div v-for="i in 3" :key="i" class="flex items-center gap-1">
                  <Skeleton class="h-9 w-[120px] mx-0.5" />
                  <div v-if="i < 3" class="h-4 w-[1px] bg-[#27272a]" />
                </div>
              </template>
              <TabsTrigger 
                v-else
                v-for="repo in repositories" 
                :key="repo.name"
                :value="repo.name"
                :title="repo.description"
                class="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-primary/10 transition-colors h-9 px-4 flex items-center gap-2 whitespace-nowrap relative"
              >
                <span class="transition-all" :class="{
                  'pr-6': repo.name === selectedRepoId,
                  'text-center w-full': repo.name !== selectedRepoId
                }">{{ repo.name }}</span>
                <button 
                  v-show="repo.name === selectedRepoId"
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400/80 hover:text-red-500 transition-colors"
                  @click.stop="removeRepository(repo.name)"
                  :title="'Remove repository'"
                >
                  <XIcon class="h-4 w-4" />
                </button>
              </TabsTrigger>
              <PopoverRoot v-model:open="showAddRepo">
                <PopoverTrigger as-child>
                  <Button
                    variant="ghost"
                    class="h-9 w-9 hover:bg-[#0a0a0a]/40 flex items-center justify-center"
                    title="Add custom repository"
                  >
                    <PlusIcon class="h-4 w-4" />
                  </Button>
                </PopoverTrigger>
                <PopoverContent 
                  class="popup-content w-[400px] p-4 rounded-md border border-[#3f3f46] !bg-[#0c0c0c] shadow-md z-10"
                  side="bottom"
                  align="end"
                  :sideOffset="5"
                  @close="showAddRepo = false"
                >
                  <h2 class="text-xl font-bold mb-4">Add Custom Repository</h2>
                  <form @submit.prevent="addCustomRepository" class="space-y-4">
                    <div>
                      <label class="block text-sm mb-1">GitHub Repository URL</label>
                      <input 
                        v-model="newRepoUrl" 
                        class="w-full p-2 rounded bg-[#27272a] border border-[#3f3f46] focus:outline-none focus:ring-1 focus:ring-[#9400f0]" 
                        placeholder="https://github.com/owner/repository"
                        required
                      >
                      <p v-if="repoUrlError" class="text-red-500 text-sm mt-1">{{ repoUrlError }}</p>
                    </div>
                    <div class="flex justify-end gap-2 mt-6">
                      <Button variant="outline" type="button" @click="showAddRepo = false">Cancel</Button>
                      <Button type="submit">Add Repository</Button>
                    </div>
                  </form>
                </PopoverContent>
              </PopoverRoot>
            </TabsList>
          </Tabs>

          <div class="flex items-center gap-2 bg-[#27272a] px-3 py-1.5 rounded-md">
            <label class="text-sm font-semibold">Nuke before flash</label>
            <Button
              :variant="nukeBeforeFlash ? 'default' : 'outline'"
              size="sm"
              @click="nukeBeforeFlash = !nukeBeforeFlash"
            >
              {{ nukeBeforeFlash ? 'On' : 'Off' }}
            </Button>
          </div>
        </div>
      </div>

      <!-- Version tabs and table -->
      <div class="flex flex-col flex-1 overflow-hidden">
        <Tabs v-model="selectedVersion" class="w-full py-2">
          <div class="w-full flex justify-center">
            <TabsList class="w-fit h-fit flex flex-wrap gap-1">
              <template v-if="loading">
                <div v-for="i in 4" :key="i" class="flex items-center gap-1">
                  <Skeleton class="h-9 w-[100px] mx-0.5" />
                  <div v-if="i < 4" class="h-4 w-[1px] bg-[#27272a]" />
                </div>
              </template>
              <template v-else>
                <TabsTrigger 
                  v-for="version in firmwareVersions" 
                  :key="version"
                  :value="version"
                  class="data-[state=active]:bg-primary data-[state=active]:text-primary-foreground hover:bg-primary/10 transition-colors"
                >
                  {{ version }}
                </TabsTrigger>
              </template>
            </TabsList>
          </div>
        </Tabs>

        <div class="flex-1 overflow-auto rounded-lg mb-18">
          <div>
            <Table class="w-full">
              <TableBody>
                <template v-if="loading">
                  <TableRow v-for="i in 5" :key="i" class="hover:bg-[#27272a]/50">
                    <TableCell class="w-full py-4">
                      <div class="flex items-center justify-between">
                        <div class="space-y-2">
                          <Skeleton class="h-5 w-[280px]" />
                          <Skeleton class="h-4 w-[180px] opacity-50" />
                        </div>
                        <Skeleton class="h-9 w-[90px]" />
                      </div>
                    </TableCell>
                  </TableRow>
                </template>
                <template v-else-if="selectedReleases.length > 0">
                  <TableRow v-for="release in selectedReleases" :key="release.name">
                    <TableCell class="font-medium">{{ release.name }}</TableCell>
                    <TableCell class="text-right">
                      <Button 
                        :variant="selectedFileUrl === release.browser_download_url ? 'default' : 'outline'" 
                        size="sm"
                        @click="selectedFileUrl = release.browser_download_url"
                      >
                        {{ selectedFileUrl === release.browser_download_url ? 'Selected' : 'Select' }}
                      </Button>
                    </TableCell>
                  </TableRow>
                </template>
              </TableBody>
            </Table>
          </div>
        </div>
      </div>
    </div>

    <!-- Status bar -->
    <StatusBar 
      :current-status="status" 
      :download-progress="downloadProgress"
      :loading="loading"
      :can-flash="!!selectedFileUrl"
      :is-flashing="isFlashing"
      @flash="flashDevice"
    />
  </div>
</template>

<style>
@import "tailwindcss";

@keyframes slideInFromTop {
  from {
    opacity: 0;
    transform: translateY(-0.5rem);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes slideOutToTop {
  from {
    opacity: 1;
    transform: translateY(0);
  }
  to {
    opacity: 0;
    transform: translateY(-0.5rem);
  }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes fadeOut {
  from { opacity: 1; }
  to { opacity: 0; }
}

[data-state="open"].popup-content {
  animation: fadeIn 120ms ease-out, slideInFromTop 120ms ease-out;
}

[data-state="closed"].popup-content {
  animation: fadeOut 120ms ease-in, slideOutToTop 120ms ease-in;
}

/* Custom scrollbar styling */
*::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

*::-webkit-scrollbar-track {
  background: #18181b;
  border-radius: 4px;
}

*::-webkit-scrollbar-thumb {
  background: #3f3f46;
  border-radius: 4px;
}

*::-webkit-scrollbar-thumb:hover {
  background: #52525b;
}

.hide-scrollbar::-webkit-scrollbar {
  display: none;
}

.hide-scrollbar {
  -ms-overflow-style: none;
  scrollbar-width: none;
}

.scrollable-container {
  padding-bottom: 32px;
}
</style>