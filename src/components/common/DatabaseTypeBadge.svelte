<script>
  import {
    DatabaseType,
    getDisplayName,
    getIconClass,
  } from "../../../utils/databaseTypes";

  export let dbType;
  export let size = "sm"; // sm, md, lg
  export let showIcon = true;
  export let showText = true;

  $: displayName = getDisplayName(dbType);
  $: iconClass = getIconClass(dbType);

  // Color mapping for badges
  $: badgeColor =
    {
      [DatabaseType.POSTGRESQL]: "primary",
      [DatabaseType.MSSQL]: "info",
      [DatabaseType.MYSQL]: "warning",
      [DatabaseType.MONGODB]: "success",
      [DatabaseType.REDIS]: "danger",
      [DatabaseType.IGNITE]: "secondary",
      [DatabaseType.MARIADB]: "warning",
      [DatabaseType.SQLITE]: "dark",
    }[dbType] || "secondary";
</script>

<span class="badge badge-{badgeColor} badge-{size}">
  {#if showIcon}
    <i class={iconClass}></i>
  {/if}
  {#if showText}
    {displayName}
  {/if}
</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .badge-sm {
    font-size: 0.75rem;
    padding: 0.2rem 0.4rem;
  }

  .badge-md {
    font-size: 0.875rem;
    padding: 0.35rem 0.6rem;
  }

  .badge-lg {
    font-size: 1rem;
    padding: 0.5rem 0.8rem;
  }
</style>
