"use client";

import { useWorkspace } from "../providers/WorkspaceProvider";

export function WorkspaceSidebar() {
  const { workspace, selectedContract, selectContract } = useWorkspace();

  if (!workspace || workspace.contracts.length <= 1) {
    return null;
  }

  return (
    <aside className="w-64 flex-shrink-0 space-y-4">
      <div className="rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 p-4 shadow-sm">
        <h3 className="text-sm font-semibold mb-3 theme-high-contrast:text-yellow-300">
          Workspace Members
        </h3>
        <nav className="space-y-1">
          {workspace.contracts.map((contract) => (
            <button
              key={contract.name}
              onClick={() => selectContract(contract.name)}
              className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                selectedContract?.name === contract.name
                  ? "bg-zinc-100 dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 font-medium"
                  : "text-zinc-500 hover:bg-zinc-50 dark:hover:bg-zinc-800/50 hover:text-zinc-700 dark:hover:text-zinc-300"
              }`}
            >
              <div className="flex justify-between items-center">
                <span className="truncate">{contract.name}</span>
                {contract.total_findings > 0 && (
                  <span className="ml-2 px-1.5 py-0.5 rounded-full bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 text-[10px] font-bold">
                    {contract.total_findings}
                  </span>
                )}
              </div>
            </button>
          ))}
        </nav>
      </div>

      {workspace.shared_libs.length > 0 && (
        <div className="rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 p-4 shadow-sm opacity-60">
          <h3 className="text-sm font-semibold mb-2">Shared Libraries</h3>
          <ul className="space-y-1">
            {workspace.shared_libs.map((lib) => (
              <li key={lib} className="px-3 py-1 text-xs text-zinc-500 truncate">
                📦 {lib}
              </li>
            ))}
          </ul>
        </div>
      )}
    </aside>
  );
}
