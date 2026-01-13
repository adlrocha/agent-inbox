/**
 * Content script for Claude.ai
 * Monitors conversations and reports task status to agent-inbox
 * Uses shared.js for common functionality
 */

console.log("Agent Inbox: Claude.ai content script loaded");
console.log("Agent Inbox: URL =", window.location.href);

// Initialize tracker with agent type
const tracker = new ConversationTracker("claude_web");

// Extract conversation ID from URL
function getConversationId() {
  const match = window.location.pathname.match(/\/chat\/([a-f0-9-]+)/);
  return match ? match[1] : null;
}

// Get conversation title (first user message or heading)
function getConversationTitle() {
  debugLog("Getting conversation title...");

  // Option 1: Look for heading or title element
  const titleSelectors = [
    '[data-testid="conversation-title"]',
    'h1',
    '[role="heading"]',
    '.text-xl',
    '.font-tiempos'
  ];

  for (const selector of titleSelectors) {
    const titleElement = document.querySelector(selector);
    if (titleElement && titleElement.textContent.trim()) {
      const title = titleElement.textContent.trim().substring(0, 100);
      debugLog("Found title via", selector, ":", title);
      return title;
    }
  }

  // Option 2: Get first user message
  const messageSelectors = [
    '[data-role="user"]',
    '[data-message-role="user"]',
    '.font-user-message',
    '[data-testid="user-message"]',
    'div[data-is-streaming="false"] p'
  ];

  for (const selector of messageSelectors) {
    const userMessages = document.querySelectorAll(selector);
    if (userMessages.length > 0) {
      const firstMessage = userMessages[0].textContent.trim();
      if (firstMessage) {
        const title = firstMessage.substring(0, 100);
        debugLog("Found title from first message via", selector, ":", title);
        return title;
      }
    }
  }

  // Option 3: Look at page title
  if (document.title && document.title !== "Claude") {
    const title = document.title.substring(0, 100);
    debugLog("Using page title:", title);
    return title;
  }

  // Option 4: Generic fallback
  debugLog("Using fallback title");
  return "Claude conversation";
}

// Check if Claude is generating a response
function isGenerating() {
  debugLog("Checking if Claude is generating...");

  // Strategy 1: Look for stop/pause button (most reliable indicator)
  const stopButtonSelectors = [
    '[aria-label="Stop generating"]',
    'button[data-testid="stop-button"]',
    'button[aria-label*="Stop"]',
    'button[aria-label*="Pause"]',
    // Additional patterns
    'button[title*="Stop"]',
    'button:has(svg) [aria-label*="stop"]',
  ];

  for (const selector of stopButtonSelectors) {
    try {
      const stopButton = document.querySelector(selector);
      if (stopButton && stopButton.offsetParent !== null && !stopButton.disabled) {
        debugLog("✓ Found visible stop button via", selector);
        return true;
      }
    } catch (e) {
      // Skip invalid selectors
    }
  }

  // Strategy 2: Look for disabled send button (indicates generating)
  const sendButtonSelectors = [
    'button[aria-label*="Send"]',
    'button[aria-label*="send"]',
    'button[type="submit"]',
    'textarea ~ button[disabled]',
    'form button[disabled]',
  ];

  for (const selector of sendButtonSelectors) {
    const buttons = document.querySelectorAll(selector);
    for (const btn of buttons) {
      if (btn.disabled && btn.offsetParent !== null) {
        debugLog("✓ Send button is disabled (generating) via", selector);
        return true;
      }
    }
  }

  // Strategy 3: Look for streaming/generating indicators
  const indicatorSelectors = [
    '[data-testid="generating"]',
    '[data-is-streaming="true"]',
    '.animate-pulse',
    '.animate-spin',
    'svg.animate-spin',
    '[class*="generating"]',
    '[class*="streaming"]',
    '[data-state="generating"]',
  ];

  for (const selector of indicatorSelectors) {
    const indicators = document.querySelectorAll(selector);
    for (const indicator of indicators) {
      const style = window.getComputedStyle(indicator);
      if (style.display !== 'none' && style.visibility !== 'hidden' && indicator.offsetParent !== null) {
        debugLog("✓ Found generating indicator via", selector);
        return true;
      }
    }
  }

  // Strategy 4: Check for disabled textarea (Claude still processing)
  const textareas = document.querySelectorAll('textarea');
  for (const textarea of textareas) {
    if (textarea.disabled && textarea.offsetParent !== null) {
      debugLog("✓ Textarea disabled (generating)");
      return true;
    }
  }

  // Strategy 5: Check document body for generating class
  if (document.body.classList.contains('generating') ||
      document.body.getAttribute('data-generating') === 'true') {
    debugLog("✓ Body has generating state");
    return true;
  }

  // Strategy 6: Check for streaming message attributes
  const messages = document.querySelectorAll('[data-message-author-role="assistant"]');
  for (const msg of messages) {
    if (msg.getAttribute('data-is-streaming') === 'true' ||
        msg.getAttribute('data-complete') === 'false') {
      debugLog("✓ Found streaming assistant message");
      return true;
    }
  }

  debugLog("✗ No generation indicators found - Claude is idle");
  return false;
}

// Check if there's a visible input textarea (Claude asking for input)
function isWaitingForUserInput() {
  // Look for input textarea that's visible and enabled
  const textareas = document.querySelectorAll('textarea');
  for (const textarea of textareas) {
    if (!textarea.disabled && textarea.offsetParent !== null) {
      // Check if it's the main input (not some other textarea)
      const placeholder = textarea.placeholder || '';
      if (placeholder.toLowerCase().includes('reply') ||
          placeholder.toLowerCase().includes('message') ||
          placeholder.toLowerCase().includes('type')) {
        debugLog("Found input textarea - waiting for user");
        return true;
      }
    }
  }

  // Look for "Continue" or similar buttons (Claude asking to continue)
  const continueButtons = document.querySelectorAll('button');
  for (const btn of continueButtons) {
    const text = btn.textContent?.trim().toLowerCase() || '';
    if ((text === 'continue' || text.includes('continue')) && btn.offsetParent !== null) {
      debugLog("Found continue button - needs user action");
      return true;
    }
  }

  return false;
}

// Main check function
function checkConversationState() {
  const conversationId = getConversationId();
  const isActive = isGenerating();
  const title = getConversationTitle();

  tracker.checkState(conversationId, isActive, title);

  // Check for waiting state (Claude-specific)
  if (!isActive && isWaitingForUserInput() && tracker.activeConversation) {
    tracker.markNeedsAttention("waiting_for_user_input");
  }
}

// URL change detection
function checkUrlChange() {
  if (tracker.checkUrlChange()) {
    // URL changed, re-check conversation state
    setTimeout(checkConversationState, 1000);
  }
}

// Helper: Diagnose Claude tracking issues
window.diagnoseClaude = function() {
  console.log("=== Claude Tracking Diagnostics ===");
  console.log("URL:", window.location.href);
  console.log("Pathname:", window.location.pathname);

  const conversationId = getConversationId();
  console.log("Conversation ID:", conversationId);

  console.log("\n=== Generation State ===");
  const isActive = isGenerating();
  console.log("Is generating:", isActive);

  const waitingForInput = isWaitingForUserInput();
  console.log("Waiting for input:", waitingForInput);

  const title = getConversationTitle();
  console.log("Conversation title:", title);

  console.log("\n=== Tracker State ===");
  console.log("Active conversation:", tracker.activeConversation);
  console.log("Is transitioning:", tracker.isTransitioning);

  console.log("\n=== DOM Elements Check ===");
  console.log("Total buttons:", document.querySelectorAll('button').length);

  // Check stop button
  const stopBtn = document.querySelector('[aria-label*="Stop"]');
  console.log("Stop button:", stopBtn ? {
    visible: stopBtn.offsetParent !== null,
    disabled: stopBtn.disabled,
    text: stopBtn.textContent?.trim()
  } : "not found");

  // Check send button
  const sendBtns = document.querySelectorAll('button[aria-label*="Send"]');
  console.log("Send buttons:", sendBtns.length);
  sendBtns.forEach((btn, i) => {
    console.log(`  Send ${i}:`, {
      disabled: btn.disabled,
      visible: btn.offsetParent !== null
    });
  });

  // Check textareas
  const textareas = document.querySelectorAll('textarea');
  console.log("Textareas:", textareas.length);
  textareas.forEach((ta, i) => {
    console.log(`  Textarea ${i}:`, {
      disabled: ta.disabled,
      visible: ta.offsetParent !== null
    });
  });

  // Check streaming indicators
  const streaming = document.querySelectorAll('[data-is-streaming="true"]');
  console.log("Streaming elements:", streaming.length);

  const animated = document.querySelectorAll('.animate-pulse, .animate-spin');
  console.log("Animated elements:", animated.length);

  console.log("\n=== Manual State Check ===");
  checkConversationState();
};

// Initialize
function initialize() {
  console.log("Agent Inbox: Initializing Claude.ai monitoring");

  // Poll for conversation state changes
  setInterval(checkConversationState, 2000);

  // Check for URL changes (SPA navigation)
  setInterval(checkUrlChange, 1000);

  // Initial check
  setTimeout(checkConversationState, 2000);
}

// Handle page unload
window.addEventListener("beforeunload", () => {
  tracker.handleUnload();
});

// Wait for page to be ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initialize);
} else {
  initialize();
}
