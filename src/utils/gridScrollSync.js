/**
 * Grid Scroll Sync Utility
 * Handles scroll synchronization between table sections
 */

import { tick } from "svelte";

/**
 * Sync column widths between header and body tables
 */
export async function syncColumnWidths(headerWrapper, tableWrapper) {
  await tick();

  if (!headerWrapper || !tableWrapper) return;

  const headerTable = headerWrapper.querySelector("table");
  const bodyTable = tableWrapper.querySelector("table");

  if (!headerTable || !bodyTable) return;

  const headerCells = headerTable.querySelectorAll("thead th");
  const bodyCells = bodyTable.querySelectorAll("tbody tr:first-child td");

  if (bodyCells.length === 0) return;

  bodyCells.forEach((bodyCell, index) => {
    if (headerCells[index]) {
      const bodyWidth = bodyCell.getBoundingClientRect().width;
      const headerWidth = headerCells[index].getBoundingClientRect().width;
      const width = Math.max(bodyWidth, headerWidth);

      headerCells[index].style.width = `${width}px`;
      headerCells[index].style.minWidth = `${width}px`;
      headerCells[index].style.maxWidth = `${width}px`;

      bodyCell.style.width = `${width}px`;
      bodyCell.style.minWidth = `${width}px`;
      bodyCell.style.maxWidth = `${width}px`;
    }
  });
}

/**
 * Check if scrolled near bottom
 */
export function isScrolledNearBottom(tableWrapper, threshold = 200) {
  if (!tableWrapper) return false;

  const scrollHeight = tableWrapper.scrollHeight;
  const clientHeight = tableWrapper.clientHeight;
  const currentScrollTop = tableWrapper.scrollTop;
  const distanceFromBottom = scrollHeight - (currentScrollTop + clientHeight);

  return distanceFromBottom < threshold;
}

/**
 * Check if scrolling down
 */
export function isScrollingDown(currentScrollTop, lastScrollTop) {
  return currentScrollTop > lastScrollTop;
}

/**
 * Should trigger load more (with conditions)
 */
export function shouldLoadMore(
  scrolledToBottom,
  hasMoreData,
  isLoadingMore,
  isScrollingDown,
  timeSinceLastLoad,
  throttleTime = 1000
) {
  return (
    scrolledToBottom &&
    hasMoreData &&
    !isLoadingMore &&
    isScrollingDown &&
    timeSinceLastLoad > throttleTime
  );
}
