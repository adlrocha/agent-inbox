/**
 * Content script for Gemini
 * Monitors conversations and reports task status to agent-inbox
 * Uses shared.js for common functionality
 */

console.log("Agent Inbox: Gemini content script loaded");
console.log("Agent Inbox: URL =", window.location.href);
console.log("Agent Inbox: Full location =", {
  href: window.location.href,
  pathname: window.location.pathname,
  search: window.location.search,
  hash: window.location.hash
});

// Initialize tracker with agent type
const tracker = new ConversationTracker("gemini_web");

// Extract conversation ID from URL
function getConversationId() {
  debugLog("Extracting conversation ID from URL:", window.location.href);
  debugLog("Pathname:", window.location.pathname);
  debugLog("Search:", window.location.search);

  // Try different URL patterns that Gemini might use
  const patterns = [
    /\/chat\/([a-zA-Z0-9_-]+)/,           // /chat/abc123
    /\/app\/([a-zA-Z0-9_-]+)/,            // /app/abc123
    /[?&]q=([a-zA-Z0-9_-]+)/,             // ?q=abc123
    /[?&]thread=([a-zA-Z0-9_-]+)/,        // ?thread=abc123
    /[?&]conversation=([a-zA-Z0-9_-]+)/,  // ?conversation=abc123
  ];

  for (const pattern of patterns) {
    const urlToCheck = window.location.pathname + window.location.search;
    const match = urlToCheck.match(pattern);
    if (match) {
      debugLog("Found conversation ID via pattern:", pattern, "=>", match[1]);
      return match[1];
    }
  }

  // If no pattern matches but we're on Gemini, use a synthetic ID based on URL
  if (window.location.href.includes('gemini.google.com')) {
    // Use pathname as ID if it's not just "/" or "/app"
    const path = window.location.pathname;
    if (path && path !== '/' && path !== '/app' && path !== '/app/') {
      const syntheticId = path.replace(/\//g, '_').substring(1) || 'home';
      debugLog("Using synthetic conversation ID:", syntheticId);
      return syntheticId;
    }
  }

  debugLog("No conversation ID found");
  return null;
}

// Get conversation title (first user message)
function getConversationTitle() {
  debugLog("Getting conversation title...");

  // Try to find the first user message
  const messageSelectors = [
    '[data-message-author-role="user"]',
    '.user-message',
    '[class*="user"]',
    '[data-role="user"]'
  ];

  for (const selector of messageSelectors) {
    const userMessages = document.querySelectorAll(selector);
    if (userMessages.length > 0) {
      const firstMessage = userMessages[0].textContent.trim();
      if (firstMessage) {
        debugLog("Found title from message via", selector);
        return firstMessage.substring(0, 100);
      }
    }
  }

  // Look for chat title
  const titleSelectors = [
    'h1',
    '[data-testid="chat-title"]',
    '.chat-title',
    '[role="heading"]'
  ];

  for (const selector of titleSelectors) {
    const titleElement = document.querySelector(selector);
    if (titleElement && titleElement.textContent.trim()) {
      debugLog("Found title via", selector);
      return titleElement.textContent.trim().substring(0, 100);
    }
  }

  // Use page title as fallback
  if (document.title && document.title !== "Gemini") {
    debugLog("Using page title");
    return document.title.substring(0, 100);
  }

  debugLog("Using fallback title");
  return "Gemini conversation";
}

// Check if Gemini is generating a response
function isGenerating() {
  // Strategy 1: Look for stop button
  const stopButtonSelectors = [
    'button[aria-label*="Stop"]',
    'button[data-testid="stop-button"]',
    'button[aria-label*="stop"]'  // Lowercase variant
  ];

  for (const selector of stopButtonSelectors) {
    const stopButton = document.querySelector(selector);
    if (stopButton && !stopButton.disabled && stopButton.offsetParent !== null) {
      debugLog("Found stop button via", selector);
      return true;
    }
  }

  // Strategy 2: Look for disabled send button (indicates generating)
  const sendButtons = document.querySelectorAll('button[aria-label*="Send"], button[type="submit"]');
  for (const btn of sendButtons) {
    if (btn.disabled && btn.offsetParent !== null) {
      debugLog("Send button is disabled (generating)");
      return true;
    }
  }

  // Strategy 3: Look for loading indicators
  const loadingSelectors = [
    '[data-testid="loading"]',
    '.loading',
    '[class*="spinner"]',
    '[class*="generating"]',
    '.animate-pulse',
    '.animate-spin',
    'svg.animate-spin'
  ];

  for (const selector of loadingSelectors) {
    const indicators = document.querySelectorAll(selector);
    for (const indicator of indicators) {
      const style = window.getComputedStyle(indicator);
      if (style.display !== "none" && style.visibility !== "hidden" && indicator.offsetParent !== null) {
        debugLog("Found loading indicator via", selector);
        return true;
      }
    }
  }

  // Strategy 4: Look for typing indicators
  const typingIndicators = document.querySelectorAll('[class*="typing"], [class*="dots"]');
  for (const indicator of typingIndicators) {
    if (indicator.offsetParent !== null) {
      debugLog("Found typing indicator");
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
}

// URL change detection
function checkUrlChange() {
  if (tracker.checkUrlChange()) {
    // URL changed, re-check conversation state
    setTimeout(checkConversationState, 1000);
  }
}

// Helper: Diagnose Gemini tracking issues
window.diagnoseGemini = function() {
  console.log("=== Gemini Tracking Diagnostics ===");
  console.log("URL:", window.location.href);
  console.log("Pathname:", window.location.pathname);
  console.log("Search:", window.location.search);

  const conversationId = getConversationId();
  console.log("Conversation ID:", conversationId);

  const isActive = isGenerating();
  console.log("Is generating:", isActive);

  const title = getConversationTitle();
  console.log("Conversation title:", title);

  console.log("Active conversation:", tracker.activeConversation);

  console.log("=== DOM Elements Check ===");
  console.log("Buttons:", document.querySelectorAll('button').length);
  console.log("Textareas:", document.querySelectorAll('textarea').length);
  console.log("Messages:", document.querySelectorAll('[data-message-author-role]').length);

  console.log("=== Try triggering check manually ===");
  checkConversationState();
};

// Initialize
function initialize() {
  console.log("Agent Inbox: Initializing Gemini monitoring");

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
