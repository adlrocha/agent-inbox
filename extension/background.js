/**
 * Background service worker for Agent Inbox extension
 * Handles native messaging with the agent-bridge host
 */

const NATIVE_HOST_NAME = "com.agent_tasks.bridge";

// Connection to native messaging host
let nativePort = null;

// Connect to native messaging host
function connectToNativeHost() {
  try {
    nativePort = chrome.runtime.connectNative(NATIVE_HOST_NAME);

    nativePort.onMessage.addListener((message) => {
      console.log("Received from native host:", message);
      // Native host can send back confirmations or errors
      if (message && message.status === "error") {
        console.error("Native host error:", message.message);
      }
    });

    nativePort.onDisconnect.addListener(() => {
      console.log("Disconnected from native host");
      const error = chrome.runtime.lastError;
      if (error) {
        console.error("Native host disconnection error:", error);
      }
      nativePort = null;

      // Don't auto-reconnect - only reconnect when actually needed
      // (This prevents error loops when extension ID isn't configured)
    });

    console.log("Connected to native host:", NATIVE_HOST_NAME);
  } catch (error) {
    console.error("Failed to connect to native host:", error);
    console.error("Make sure:");
    console.error("1. agent-bridge is installed: /usr/local/bin/agent-bridge");
    console.error("2. Native messaging manifest is configured with correct extension ID");
    console.error("3. Manifest location: ~/.config/google-chrome/NativeMessagingHosts/com.agent_tasks.bridge.json");
    nativePort = null;
  }
}

// Send message to native host
function sendToNativeHost(message) {
  if (!nativePort) {
    console.warn("Native port not connected, attempting to connect...");
    connectToNativeHost();

    // Try to send after a short delay
    setTimeout(() => {
      if (nativePort) {
        try {
          nativePort.postMessage(message);
          console.log("Sent to native host (after reconnect):", message);
        } catch (error) {
          console.error("Failed to send after reconnect:", error);
        }
      } else {
        console.error("Could not reconnect to native host. Message dropped:", message);
      }
    }, 500);
    return;
  }

  try {
    nativePort.postMessage(message);
    console.log("Sent to native host:", message);
  } catch (error) {
    console.error("Failed to send to native host:", error);
    nativePort = null;
  }
}

// Handle messages from content scripts
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  console.log("Received from content script:", message);

  if (message.type === "task_update") {
    // Forward to native host
    sendToNativeHost(message);
    sendResponse({ status: "ok" });
  } else if (message.type === "ping") {
    sendResponse({ status: "ok", connected: nativePort !== null });
  }

  return true; // Keep message channel open for async response
});

// Initialize connection on startup
connectToNativeHost();

console.log("Agent Inbox background service worker initialized");
