import { DatabaseType } from "./databaseTypes";

/**
 * Get tab configuration based on database type
 */
export function getTabsForDatabase(dbType) {
  if (dbType === DatabaseType.MSSQL) {
    return [
      { id: "Columns", label: "Columns", icon: "fas fa-columns" },
      { id: "Keys", label: "Keys", icon: "fas fa-key" },
      { id: "Foreign Keys", label: "Foreign Keys", icon: "fas fa-link" },
      { id: "Indexes", label: "Indexes", icon: "fas fa-sort-amount-down" },
      { id: "References", label: "References", icon: "fas fa-sitemap" },
      { id: "Triggers", label: "Triggers", icon: "fas fa-bolt" },
      { id: "Statistics", label: "Statistics", icon: "fas fa-chart-bar" },
      { id: "DDL", label: "DDL", icon: "fas fa-code" },
      { id: "Virtual", label: "Virtual", icon: "fas fa-cube" },
    ];
  }

  if (dbType === DatabaseType.POSTGRESQL) {
    return [
      { id: "Columns", label: "Columns", icon: "fas fa-columns" },
      { id: "Constraints", label: "Constraints", icon: "fas fa-lock" },
      { id: "Foreign Keys", label: "Foreign Keys", icon: "fas fa-key" },
      { id: "Indexes", label: "Indexes", icon: "fas fa-sort-amount-down" },
      {
        id: "Dependencies",
        label: "Dependencies",
        icon: "fas fa-project-diagram",
      },
      { id: "References", label: "References", icon: "fas fa-link" },
      { id: "Partitions", label: "Partitions", icon: "fas fa-th-large" },
      { id: "Child tables", label: "Child tables", icon: "fas fa-table" },
      { id: "Triggers", label: "Triggers", icon: "fas fa-bolt" },
      { id: "Rules", label: "Rules", icon: "fas fa-gavel" },
      { id: "Policies", label: "Policies", icon: "fas fa-shield-alt" },
      { id: "Statistics", label: "Statistics", icon: "fas fa-chart-bar" },
      { id: "Permissions", label: "Permissions", icon: "fas fa-user-lock" },
      { id: "DDL", label: "DDL", icon: "fas fa-code" },
      { id: "Virtual", label: "Virtual", icon: "fas fa-cube" },
    ];
  }

  // MySQL and other databases (default)
  return [
    { id: "Columns", label: "Columns", icon: "fas fa-columns" },
    { id: "Constraints", label: "Constraints", icon: "fas fa-lock" },
    { id: "Foreign Keys", label: "Foreign Keys", icon: "fas fa-key" },
    { id: "References", label: "References", icon: "fas fa-link" },
    { id: "Triggers", label: "Triggers", icon: "fas fa-bolt" },
    { id: "Indexes", label: "Indexes", icon: "fas fa-sort-amount-down" },
    { id: "Partitions", label: "Partitions", icon: "fas fa-th-large" },
    { id: "Statistics", label: "Statistics", icon: "fas fa-chart-bar" },
    { id: "DDL", label: "DDL", icon: "fas fa-code" },
    { id: "Virtual", label: "Virtual", icon: "fas fa-cube" },
  ];
}
