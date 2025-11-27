/**
 * Application initialization service
 */
export async function initializeApplication(onProgress) {
  const steps = [
    { message: "Loading configuration", duration: 300 },
    { message: "Initializing database drivers", duration: 400 },
    { message: "Loading saved connections", duration: 300 },
    { message: "Preparing workspace", duration: 200 },
    { message: "Ready", duration: 100 },
  ];

  let currentProgress = 0;
  const progressStep = 100 / steps.length;

  for (let i = 0; i < steps.length; i++) {
    // Update HTML splash screen
    updateHtmlSplash(currentProgress, steps[i].message);

    if (onProgress) {
      onProgress({
        progress: currentProgress,
        message: steps[i].message,
      });
    }

    await new Promise((resolve) => setTimeout(resolve, steps[i].duration));
    currentProgress += progressStep;
  }

  // Final progress update
  updateHtmlSplash(100, "Ready");

  if (onProgress) {
    onProgress({
      progress: 100,
      message: "Ready",
    });
  }

  // Small delay before hiding splash
  await new Promise((resolve) => setTimeout(resolve, 300));

  // Hide the initial HTML splash screen
  hideInitialSplash();

  return true;
}

/**
 * Update the HTML splash screen progress
 */
function updateHtmlSplash(progress, message) {
  const progressFill = document.querySelector("#initial-splash .progress-fill");
  const progressText = document.querySelector("#initial-splash .progress-text");
  const loadingMessage = document.querySelector(
    "#initial-splash .loading-message"
  );

  if (progressFill) {
    progressFill.style.animation = "none";
    progressFill.style.width = `${progress}%`;
  }
  if (progressText) {
    progressText.textContent = `${Math.round(progress)}%`;
  }
  if (loadingMessage) {
    loadingMessage.textContent = message;
  }
}

/**
 * Hide the initial splash screen from index.html
 */
function hideInitialSplash() {
  const initialSplash = document.getElementById("initial-splash");
  if (initialSplash) {
    initialSplash.classList.add("hidden");
    // Remove from DOM after transition
    setTimeout(() => {
      initialSplash.remove();
    }, 300);
  }
}
