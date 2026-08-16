import { useState } from "react"
import { Link } from "react-router-dom"
import { useQuery } from "urql"

import { CreateProjectModal } from "@/components/projects/CreateProjectModal"
import { Button } from "@/components/ui/button"
import { EmptyState } from "@/components/ui-states/EmptyState"
import { ErrorState } from "@/components/ui-states/ErrorState"
import { LoadingSpinner } from "@/components/ui-states/LoadingSpinner"

type Project = {
  id: string
  name: string
}

type ProjectsQueryData = {
  projects: Project[]
}

const PROJECTS_QUERY = `
  query Projects {
    projects {
      id
      name
    }
  }
`

function ProjectsPage() {
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const [{ data, fetching, error }, reexecuteQuery] = useQuery<ProjectsQueryData>({
    query: PROJECTS_QUERY,
  })

  const handleRetry = () => {
    reexecuteQuery({ requestPolicy: "network-only" })
  }

  const handleCreated = () => {
    setIsCreateModalOpen(false)
    reexecuteQuery({ requestPolicy: "network-only" })
  }

  return (
    <div>
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">プロジェクト</h1>
        <Button onClick={() => setIsCreateModalOpen(true)}>+ 新規作成</Button>
      </div>

      <div className="mt-6">
        {fetching && <LoadingSpinner message="読み込み中..." />}

        {!fetching && error && (
          <ErrorState message="プロジェクトの取得に失敗しました" onRetry={handleRetry} />
        )}

        {!fetching && !error && data && data.projects.length === 0 && (
          <EmptyState
            message="まだプロジェクトがありません"
            action={<Button onClick={() => setIsCreateModalOpen(true)}>+ 新規作成</Button>}
          />
        )}

        {!fetching && !error && data && data.projects.length > 0 && (
          <ul className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {data.projects.map((project) => (
              <li key={project.id}>
                <Link
                  to={`/projects/${project.id}`}
                  className="block rounded-lg border border-border bg-card p-4 text-card-foreground transition-colors hover:bg-muted"
                >
                  <span className="font-medium">{project.name}</span>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </div>

      <CreateProjectModal
        open={isCreateModalOpen}
        onOpenChange={setIsCreateModalOpen}
        onCreated={handleCreated}
      />
    </div>
  )
}

export default ProjectsPage
