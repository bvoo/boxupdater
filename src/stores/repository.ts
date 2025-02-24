import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Repository } from '../lib/repositories'

const DEFAULT_REPOSITORIES: Repository[] = [
  {
    name: "HayBox",
    owner: "JonnyHaystack",
    description: "HayBox",
    asset_filter: "\\.uf2$",
  },
  {
    name: "GP2040-CE",
    owner: "OpenStickCommunity",
    description: "GP2040-CE Firmware",
    asset_filter: "\\.uf2$",
  },
  {
    name: "pico-rectangle",
    owner: "JulienBernard3383279",
    description: "Pico Rectangle",
    asset_filter: "\\.uf2$",
  },
  {
    name: "HayBox-GRAM",
    owner: "GRAMCTRL",
    description: "HayBox-GRAM",
    asset_filter: "\\.uf2$",
  }
]

export const useRepositoryStore = defineStore('repository', () => {
  const repositories = ref<Repository[]>([])
  const storeKey = 'boxupdater-repositories'

  function loadRepositories() {
    const stored = localStorage.getItem(storeKey)
    if (stored) {
      repositories.value = JSON.parse(stored)
    } else {
      repositories.value = DEFAULT_REPOSITORIES
      saveRepositories()
    }
    return repositories.value
  }

  function saveRepositories() {
    localStorage.setItem(storeKey, JSON.stringify(repositories.value))
  }

  function addRepository(repository: Repository) {
    // Generate unique name if it already exists
    let uniqueName = repository.name
    let counter = 1
    while (repositories.value.some(r => r.name === uniqueName)) {
      uniqueName = `${repository.name}-${counter}`
      counter++
    }

    repository.name = uniqueName
    repositories.value.push(repository)
    saveRepositories()
  }

  function removeRepository(name: string) {
    const index = repositories.value.findIndex(r => r.name === name)
    if (index !== -1) {
      repositories.value.splice(index, 1)
      saveRepositories()
    }
  }

  // Load repositories on store creation
  loadRepositories()

  return {
    repositories,
    addRepository,
    removeRepository,
    loadRepositories
  }
})