"use client";

import React, { createContext, useContext, useState, useCallback, useMemo } from "react";
import type { WorkspaceSummary, WorkspaceMember, AnalysisReport } from "../types";

interface WorkspaceContextType {
  workspace: WorkspaceSummary | null;
  selectedContract: WorkspaceMember | null;
  setWorkspace: (w: WorkspaceSummary | null) => void;
  selectContract: (name: string) => void;
  updateContractReport: (name: string, report: AnalysisReport) => void;
}

const WorkspaceContext = createContext<WorkspaceContextType | undefined>(undefined);

export function WorkspaceProvider({ children }: { children: React.ReactNode }) {
  const [workspace, setWorkspaceState] = useState<WorkspaceSummary | null>(null);
  const [selectedContractName, setSelectedContractName] = useState<string | null>(null);

  const setWorkspace = useCallback((w: WorkspaceSummary | null) => {
    setWorkspaceState(w);
    if (w && w.contracts.length > 0) {
      setSelectedContractName(w.contracts[0].name);
    } else {
      setSelectedContractName(null);
    }
  }, []);

  const selectContract = useCallback((name: string) => {
    setSelectedContractName(name);
  }, []);

  const updateContractReport = useCallback((name: string, report: AnalysisReport) => {
    setWorkspaceState((prev) => {
      if (!prev) return null;
      return {
        ...prev,
        contracts: prev.contracts.map((c) =>
          c.name === name ? { ...c, report } : c
        ),
      };
    });
  }, []);

  const selectedContract = useMemo(() => {
    if (!workspace || !selectedContractName) return null;
    return workspace.contracts.find((c) => c.name === selectedContractName) || null;
  }, [workspace, selectedContractName]);

  const value = useMemo(
    () => ({
      workspace,
      selectedContract,
      setWorkspace,
      selectContract,
      updateContractReport,
    }),
    [workspace, selectedContract, setWorkspace, selectContract, updateContractReport]
  );

  return <WorkspaceContext.Provider value={value}>{children}</WorkspaceContext.Provider>;
}

export function useWorkspace() {
  const context = useContext(WorkspaceContext);
  if (context === undefined) {
    throw new Error("useWorkspace must be used within a WorkspaceProvider");
  }
  return context;
}
