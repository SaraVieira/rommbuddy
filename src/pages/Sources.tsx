import { useEffect } from "react"
import LocalSourceSection from "../components/sources/LocalSourceSection"
import RommSourceSection from "../components/sources/RommSourceSection"
import { useSetAtom } from "jotai"
import { loadSourcesAtom } from "@/store/sources"

export default function Sources() {
  const loadSources = useSetAtom(loadSourcesAtom)

  useEffect(() => {
    loadSources()
  }, [loadSources])

  return (
    <div className="page">
      <div className="flex flex-col gap-xs mb-xl">
        <h1 className="font-display text-page-title font-bold text-text-primary uppercase">
          Sources
        </h1>
        <span className="text-nav text-text-muted">
          Manage your ROM sources and sync connections.
        </span>
      </div>

      <LocalSourceSection onReload={loadSources} />

      <RommSourceSection onReload={loadSources} />
    </div>
  )
}
