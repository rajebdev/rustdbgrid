<script>
  import { onMount } from "svelte";

  export let progress = 0;
  export let message = "Loading...";
  export let show = true;

  let dots = "";
  let dotInterval;

  onMount(() => {
    // Animated dots effect
    dotInterval = setInterval(() => {
      dots = dots.length >= 3 ? "" : dots + ".";
    }, 500);

    return () => {
      if (dotInterval) clearInterval(dotInterval);
    };
  });
</script>

{#if show}
  <div class="splash-screen">
    <div class="splash-content">
      <!-- Logo/Icon -->
      <div class="splash-logo">
        <i class="fas fa-database"></i>
      </div>

      <!-- App Name -->
      <h1 class="splash-title">RustDBGrid</h1>
      <p class="splash-subtitle">Universal Database Manager</p>

      <!-- Progress Bar -->
      <div class="progress-container">
        <div class="progress-bar">
          <div class="progress-fill" style="width: {progress}%"></div>
        </div>
        <div class="progress-text">{Math.round(progress)}%</div>
      </div>

      <!-- Loading Message -->
      <div class="loading-message">
        {message}{dots}
      </div>

      <!-- Spinner -->
      <div class="spinner-container">
        <div class="spinner"></div>
      </div>
    </div>
  </div>
{/if}

<style>
  .splash-screen {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    animation: fadeIn 0.3s ease-in;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .splash-content {
    text-align: center;
    color: white;
    max-width: 400px;
    padding: 2rem;
  }

  .splash-logo {
    font-size: 80px;
    margin-bottom: 1.5rem;
    animation: pulse 2s ease-in-out infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      transform: scale(1);
      opacity: 1;
    }
    50% {
      transform: scale(1.1);
      opacity: 0.8;
    }
  }

  .splash-title {
    font-size: 2.5rem;
    font-weight: 700;
    margin: 0;
    margin-bottom: 0.5rem;
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.3);
  }

  .splash-subtitle {
    font-size: 1rem;
    margin: 0;
    margin-bottom: 2rem;
    opacity: 0.9;
    font-weight: 300;
  }

  .progress-container {
    margin-bottom: 1.5rem;
  }

  .progress-bar {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.2);
    border-radius: 10px;
    overflow: hidden;
    margin-bottom: 0.5rem;
    box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .progress-fill {
    height: 100%;
    background: linear-gradient(90deg, #4facfe 0%, #00f2fe 100%);
    border-radius: 10px;
    transition: width 0.3s ease-out;
    box-shadow: 0 0 10px rgba(79, 172, 254, 0.5);
  }

  .progress-text {
    font-size: 0.875rem;
    font-weight: 600;
    opacity: 0.9;
  }

  .loading-message {
    font-size: 1rem;
    margin-bottom: 1.5rem;
    min-height: 1.5rem;
    font-weight: 400;
    opacity: 0.95;
  }

  .spinner-container {
    display: flex;
    justify-content: center;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 4px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
