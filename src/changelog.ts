export interface ChangelogRelease {
  version: string;
  date: string;
  notes: string[];
}

export const CHANGELOG: ChangelogRelease[] = [
  {
    version: "1.0.4",
    date: "September 16, 2026",
    notes: [
      "Commit can amend the last commit. The checkbox fills in that message; a warning appears if it is already on the remote.",
      "Repository rows sit the branch icon closer to the branch name.",
      "When a pull leaves merge or rebase conflicts, the files list shows them with Open in the chosen editor, Mark resolved, Continue, and Abort. Repository rows show a conflict count.",
      "General settings pick the editor used on conflicted files, or the system default. The button uses that name.",
      "Git settings edit your global git config: name, email, default branch, pull.rebase, checkout.defaultRemote, and the config file itself.",
      "The repository terminal pane can be resized vertically. The height is saved in settings.",
      "Reset buttons restore the default files pane width and terminal height in General settings.",
      "File list paths stay left-aligned; only the directory portion truncates when space is tight.",
      "Expand and Collapse in the repositories toolbar are hidden until at least one group exists.",
    ],
  },
  {
    version: "1.0.3",
    date: "September 16, 2026",
    notes: [
      "Optional active hours for auto-refresh in Schedule settings. Off by default; Business (8am–6pm) and Personal (6am–11pm) presets, or custom times. Automatic fetches pause outside the window.",
      "Pull next to Refresh pulls the current branch for every standalone and grouped repository.",
      "Terminal on the open-repo toolbar opens a shell in the bottom half of the graph, branches, or stashes view, started in that repository.",
    ],
  },
  {
    version: "1.0.2",
    date: "September 15, 2026",
    notes: [
      "Branches view lists names immediately, then fills Merged/Partial in place. The list stays in the same order, and a spinner shows while leftover work is still being checked.",
      "Delete merged reuses that list and removes leftovers in one git call.",
    ],
  },
  {
    version: "1.0.1",
    date: "September 15, 2026",
    notes: [
      "Check for Updates in Settings, the Shipyard menu, or automatically on launch. Newer builds download from GitHub Releases and install in place.",
      "A green update button appears in the tab bar when a newer build is available.",
      "Settings has a left-hand menu: Updates, then General, Window, and the JSON file. The settings icon opens General.",
    ],
  },
  {
    version: "1.0.0",
    date: "September 15, 2026",
    notes: [
      "GitHub icon in the status bar opens the public repository.",
      "First official release.",
      "Click a commit in the graph to see the files it changed and open a diff.",
      "Stashes view on the repo toolbar to apply, pop, or drop saved changes. Stash working-tree changes from the files pane or that view.",
      "Branches view has checkout and rename buttons on each local branch.",
      "Settings tab has preferences for auto-refresh, diff layout, files pane width, and window size and position. Open settings.json from the top-right icon to edit, export, or import the full file.",
      "App data lives in settings.json. An existing groups.json is migrated on launch.",
      "Drag handles reorder group repos, standalone repos, or the groups themselves. Sort A–Z in the repositories toolbar sorts groups and standalone repos; the group menu sorts that group’s repos.",
      "Standalone repositories are spaced like groups, with a color and optional label from the row menu. Each row has pull, checkout, and refresh.",
      "Pull on groups, standalone repos, and the open-repo toolbar runs the current branch. The caret opens the existing branch-options modal.",
      "Repository rows and open repo tabs show a repo icon.",
      "Checking out a branch from the Branches view updates the current branch immediately.",
      "Open-repo toolbar is two lines: identity on top, git actions and view toggles below.",
      "Branches view marks leftover work as Merged or Partial, and delete-merged asks before force-deleting branches git will not remove safely.",
    ],
  },
  {
    version: "0.2.0",
    date: "September 14, 2026",
    notes: [
      "Branches view for leftover local work: merged vs still unique, per-branch delete, and delete-merged that never force-deletes or removes develop, main, master, or the current branch.",
      "History tab logs every git command Shipyard runs, with a commands/full-detail toggle, hide-status filter, and a clear action.",
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
