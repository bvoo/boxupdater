import { invoke } from '@tauri-apps/api/core'

export interface Release {
  name: string
  tag_name: string
  browser_download_url: string
}

export interface Repository {
  name: string
  owner: string
  description: string
  asset_filter: string
}

export async function fetchReleasesByRepo(repoName: string): Promise<Release[]> {
  return invoke('get_releases', { repoName })
}

export async function getRepositories(): Promise<Repository[]> {
  return invoke('get_repositories')
}