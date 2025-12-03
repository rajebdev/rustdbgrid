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
      const width = bodyCell.getBoundingClientRect().width;
      headerCells[index].style.width = `${width}px`;
      headerCells[index].style.minWidth = `${width}px`;
      headerCells[index].style.maxWidth = `${width}px`;
    }
  });
}

/**
 * Handle wheel scroll with custom step
 */
export function handleWheel(
  event,
  tableWrapper,
  rowNumbersWrapper,
  scrollStep = 0
) {
  // If scroll step is 0 or not set, use browser default
  if (!scrollStep || scrollStep <= 0) return;

  event.preventDefault();

  const direction = event.deltaY > 0 ? 1 : -1;
  const scrollAmount = direction * scrollStep;

  if (tableWrapper) {
    tableWrapper.scrollTop += scrollAmount;
    if (rowNumbersWrapper) {
      rowNumbersWrapper.scrollTop = tableWrapper.scrollTop;
    }
  }
}

/**
 * Sync scroll positions between table sections
 */
export function syncScrollPositions(
  tableWrapper,
  headerWrapper,
  rowNumbersWrapper,
  currentScrollLeft,
  currentScrollTop
) {
  if (headerWrapper) {
    headerWrapper.scrollLeft = currentScrollLeft;
  }

  if (rowNumbersWrapper) {
    rowNumbersWrapper.scrollTop = currentScrollTop;
  }
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

/**
 * Get scroll distance from bottom
 */
export function getScrollDistanceFromBottom(tableWrapper) {
  if (!tableWrapper) return 0;

  const scrollHeight = tableWrapper.scrollHeight;
  const clientHeight = tableWrapper.clientHeight;
  const currentScrollTop = tableWrapper.scrollTop;

  return scrollHeight - (currentScrollTop + clientHeight);
}
