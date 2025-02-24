import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { Repository } from '../lib/repositories'

const DEFAULT_REPOSITORIES: Repository[] = [
  {
    name: "HayBox",
    displayName: "HayBox",
    owner: "JonnyHaystack",
    description: "HayBox",
    asset_filter: "\\.uf2$",
  },
  {
    name: "GP2040-CE",
    displayName: "GP2040-CE",
    owner: "OpenStickCommunity",
    description: "GP2040-CE Firmware",
    asset_filter: "\\.uf2$",
  },
  {
    name: "pico-rectangle",
    displayName: "Pico Rectangle",
    owner: "JulienBernard3383279",
    description: "Pico Rectangle",
    asset_filter: "\\.uf2$",
  },
  {
    name: "HayBox-GRAM",
    displayName: "HayBox-GRAM",
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
      // Ensure all repos have displayName (migration)
      repositories.value = repositories.value.map(repo => ({
        ...repo,
        displayName: repo.displayName || repo.name
      }))
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
    // Find repositories with the same base name
    const sameNameRepos = repositories.value.filter(r => r.name === repository.name)
    
    if (sameNameRepos.length > 0) {
      // Add owner name to all repos with same name, including the new one
      sameNameRepos.forEach(repo => {
        repo.displayName = `${repo.name} (${repo.owner})`
      })
      repository.displayName = `${repository.name} (${repository.owner})`
    }

    repositories.value.push({
      ...repository,
      displayName: repository.displayName || repository.name
    })
    saveRepositories()
  }

  function removeRepository(displayName: string) {
    const index = repositories.value.findIndex(r => r.displayName === displayName)
    if (index !== -1) {
      const removed = repositories.value.splice(index, 1)[0]
      
      // If this was part of a duplicate set, check if we need to simplify other repos' display names
      const sameNameRepos = repositories.value.filter(r => r.name === removed.name)
      if (sameNameRepos.length === 1) {
        // Only one left, can remove the owner from display name
        sameNameRepos[0].displayName = sameNameRepos[0].name
      }
      
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