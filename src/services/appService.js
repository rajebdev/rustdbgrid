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
  if (onProgress) {
    onProgress({
      progress: 100,
      message: "Ready",
    });
  }

  // Small delay before hiding splash
  await new Promise((resolve) => setTimeout(resolve, 300));

  return true;
}
