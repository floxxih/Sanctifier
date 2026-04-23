"use client";

import Link from "next/link";
import React from "react";

interface DashboardHeaderProps {
  jsonInput: string;
  setJsonInput: (v: string) => void;
  loadReport: () => void;
  handleFileUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handleContractUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
  exportToPdf: () => void;
  hasData: boolean;
  isProcessing: boolean;
  uploadStatus: string | null;
  error: string | null;
  sampleJson: string;
}

export function DashboardHeader({
  jsonInput,
  setJsonInput,
  loadReport,
  handleFileUpload,
  handleContractUpload,
  exportToPdf,
  hasData,
  isProcessing,
  uploadStatus,
  error,
  sampleJson,
}: DashboardHeaderProps) {
  return (
    <section className="rounded-xl border border-zinc-200 dark:border-zinc-800 theme-high-contrast:border-white bg-white dark:bg-zinc-900 theme-high-contrast:bg-black p-6 shadow-sm">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold theme-high-contrast:text-yellow-300">Load Analysis Report</h2>
        <Link 
          href="/dashboard/webhooks" 
          className="flex items-center gap-2 text-xs font-bold text-zinc-500 hover:text-emerald-500 transition-colors bg-zinc-50 dark:bg-zinc-950 px-3 py-1.5 rounded-lg border border-zinc-200 dark:border-zinc-800"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>
          Manage Webhooks
        </Link>
      </div>
      <p className="text-sm text-zinc-600 dark:text-zinc-400 theme-high-contrast:text-white mb-4">
        Paste JSON from <code className="bg-zinc-100 dark:bg-zinc-800 theme-high-contrast:bg-zinc-900 px-1 rounded">sanctifier analyze --format json</code>, upload an existing report, or analyze a Rust contract source file.
      </p>
      <div className="flex flex-wrap gap-2 sm:gap-4">
        <label className="flex-1 sm:flex-none text-center cursor-pointer rounded-lg border border-zinc-300 dark:border-zinc-600 theme-high-contrast:border-white px-4 py-2 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-800 theme-high-contrast:hover:bg-zinc-900 focus-within:outline-none focus-within:ring-2 focus-within:ring-zinc-400 focus-within:ring-offset-2">
          Upload JSON
          <input
            type="file"
            accept=".json"
            className="hidden"
            aria-label="JSON report file"
            data-testid="json-upload-input"
            onChange={handleFileUpload}
          />
        </label>
        <label className="flex-1 sm:flex-none text-center cursor-pointer rounded-lg border border-zinc-300 dark:border-zinc-600 theme-high-contrast:border-white px-4 py-2 text-sm hover:bg-zinc-100 dark:hover:bg-zinc-800 theme-high-contrast:hover:bg-zinc-900 focus-within:outline-none focus-within:ring-2 focus-within:ring-zinc-400 focus-within:ring-offset-2">
          {isProcessing ? "Processing..." : "Upload Contract"}
          <input
            type="file"
            accept=".rs"
            className="hidden"
            aria-label="Contract file"
            data-testid="contract-upload-input"
            onChange={handleContractUpload}
          />
        </label>
        <button
          onClick={loadReport}
          className="flex-1 sm:flex-none rounded-lg bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 theme-high-contrast:bg-white theme-high-contrast:text-black px-4 py-2 text-sm font-medium hover:bg-zinc-800 dark:hover:bg-zinc-200 theme-high-contrast:hover:bg-zinc-300 focus:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 focus-visible:ring-offset-2"
        >
          Parse JSON
        </button>
        <button
          onClick={exportToPdf}
          disabled={!hasData}
          className="flex-1 sm:flex-none rounded-lg border border-zinc-300 dark:border-zinc-600 theme-high-contrast:border-white px-4 py-2 text-sm disabled:opacity-50 hover:bg-zinc-100 dark:hover:bg-zinc-800 theme-high-contrast:hover:bg-zinc-900 focus:outline-none focus-visible:ring-2 focus-visible:ring-zinc-400 focus-visible:ring-offset-2 disabled:focus-visible:ring-0"
        >
          Export PDF
        </button>
      </div>
      {uploadStatus && (
        <p className="mt-2 text-sm text-emerald-600 dark:text-emerald-400" role="status" aria-live="polite">
          {uploadStatus}
        </p>
      )}
      {error && (
        <p className="mt-2 text-sm text-red-600 dark:text-red-400">{error}</p>
      )}
      <textarea
        value={jsonInput}
        onChange={(e) => setJsonInput(e.target.value)}
        placeholder={sampleJson}
        disabled={isProcessing}
        className="mt-4 w-full h-32 rounded-lg border border-zinc-300 dark:border-zinc-600 bg-white dark:bg-zinc-950 p-3 font-mono text-sm focus:ring-2 focus:ring-zinc-400 dark:focus:ring-zinc-600 outline-none disabled:opacity-50"
      />
    </section>
  );
}
