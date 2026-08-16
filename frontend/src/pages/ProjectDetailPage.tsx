import { Link, useParams } from "react-router-dom"
import { useQuery } from "urql"

import { ErrorState } from "@/components/ui-states/ErrorState"
import { LoadingSpinner } from "@/components/ui-states/LoadingSpinner"

type Project = {
  id: string
  name: string
}

type ProjectQueryData = {
  project: Project | null
}

type ProjectQueryVariables = {
  id: string
}

const PROJECT_QUERY = `
  query Project($id: ID!) {
    project(id: $id) {
      id
      name
    }
  }
`

function ProjectDetailPage() {
  const { id } = useParams<{ id: string }>()
  const [{ data, fetching, error }, reexecuteQuery] = useQuery<
    ProjectQueryData,
    ProjectQueryVariables
  >({
    query: PROJECT_QUERY,
    variables: { id: id ?? "" },
    pause: !id,
  })

  const handleRetry = () => {
    reexecuteQuery({ requestPolicy: "network-only" })
  }

  return (
    <div>
      <Link
        to="/projects"
        className="text-sm text-muted-foreground transition-colors hover:text-foreground"
      >
        ← 戻る
      </Link>

      <div className="mt-6">
        {fetching && <LoadingSpinner message="読み込み中..." />}

        {!fetching && error && (
          <ErrorState message="プロジェクトの取得に失敗しました" onRetry={handleRetry} />
        )}

        {!fetching && !error && data && !data.project && (
          <ErrorState message="指定されたプロジェクトが見つかりません" />
        )}

        {!fetching && !error && data?.project && (
          <h1 className="text-2xl font-bold">{data.project.name}</h1>
        )}
      </div>
    </div>
  )
}

export default ProjectDetailPage
