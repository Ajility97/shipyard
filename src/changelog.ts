export interface ChangelogRelease {
  version: string;
  date: string;
  notes: string[];
}

export const CHANGELOG: ChangelogRelease[] = [
  {
    version: "1.0.0",
    date: "September 15, 2026",
    notes: [
      "First official release.",
      "Click a commit in the graph to see the files it changed and open a diff.",
      "Stash working-tree changes, then apply, pop, or drop them from the repo toolbar.",
      "Settings tab has preferences for auto-refresh, diff layout, files pane width, and window size and position. Open settings.json from the top-right icon to edit, export, or import the full file.",
      "App data lives in settings.json. An existing groups.json is migrated on launch.",
      "Drag handles reorder group repos or the groups themselves. Sort A–Z is in the group menu and the repositories toolbar.",
    ],
  },
  {
    version: "0.2.0",
    date: "September 14, 2026",
    notes: [
      "Branches view for leftover local work: merged vs still unique, per-branch delete, and delete-merged that never force-deletes or removes develop, main, master, or the current branch.",
      "History tab logs every git command Krakdown runs, with a commands/full-detail toggle, hide-status filter, and a clear action.",
      "Settings can be exported to a JSON file from the editor.",
      "Group headers show one active action at a time (refresh, pull, or checkout) with per-repo progress, the real branch name, and a cancel control.",
      "Refresh runs a bounded pool of fetches so a group updates in waves instead of one repo at a time. All row spinners appear together.",
      "Commit graph stays in a capped left rail and tightens lane spacing on busy histories, so subjects and authors stay readable.",
      "Branches page opens immediately. Merge checks run off the UI thread and use one --merged scan plus cherry only for leftover branches.",
      "Change Log tab, opened from the version number in the status bar.",
    ],
  },
  {
    version: "0.1.0",
    date: "September 11, 2026",
    notes: [
      "First release: a local-first macOS Git client that shells out to the system git binary. No in-app login.",
      "Repository groups and standalone repos, persisted at groups.json.",
      "Live branch, ahead/behind, and working-tree counts on each repo row.",
      "Pull and checkout across a group, with a pull-from branch and checkout fallbacks.",
      "Repo tabs with a commit graph, working tree (stage, unstage, discard), inline or side-by-side diffs, and commit.",
      "Repo toolbar: branch switcher, new branch, pull, push, and a Changes toggle.",
      "JSON settings editor, window position restore, group header colors on tabs, and keyboard tab close.",
    ],
  },
];
