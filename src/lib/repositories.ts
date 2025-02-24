import { useRepositoryStore } from '../stores/repository'

export interface Release {
  name: string
  tag_name: string
  browser_download_url: string
  download_count: number
  uploaded_at: string
}

export interface Repository {
  name: string          // Original repository name
  displayName: string   // Name shown in UI (with suffix if duplicate)
  owner: string
  description: string
  asset_filter: string
}

export async function fetchReleasesByRepo(repoName: string): Promise<Release[]> {
  const store = useRepositoryStore()
  const repo = store.repositories.find(r => r.displayName === repoName)
  if (!repo) throw new Error('Repository not found')

  const url = `https://api.github.com/repos/${repo.owner}/${repo.name}/releases`
  const response = await fetch(url, {
    headers: {
      'User-Agent': 'boxupdater'
    }
  })
  
  const releases = await response.json()
  const regex = new RegExp(repo.asset_filter)

  return releases.flatMap((release: any) => 
    release.assets
      .filter((asset: any) => regex.test(asset.name))
      .map((asset: any) => ({
        name: asset.name,
        tag_name: release.tag_name,
        browser_download_url: asset.browser_download_url,
        download_count: asset.download_count,
        uploaded_at: asset.created_at
      }))
  )
}