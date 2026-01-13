/**
 * Shared utilities for content scripts
 * Used by both claude.js and gemini.js
 */

// Enable debug logging (set to false to disable)
const DEBUG = true;

function debugLog(...args) {
  if (DEBUG) {
    console.log("[Agent Inbox DEBUG]", ...args);
  }
}

// Generate a UUID v4
function generateUUID() {
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

// Send task update to background script
function sendTaskUpdate(agentType, taskId, status, title, context) {
  const message = {
    type: "task_update",
    task_id: taskId,
    agent_type: agentType,
    status: status,
    title: title,
    context: context,
  };

  chrome.runtime.sendMessage(message, (response) => {
    if (chrome.runtime.lastError) {
      console.error("Failed to send task update:", chrome.runtime.lastError);
    } else {
      console.log("Task update sent:", status, taskId);
    }
  });
}

// Helper: Inspect DOM to find buttons (run in console: inspectButtons())
window.inspectButtons = function() {
  const buttons = document.querySelectorAll('button');
  console.log("All buttons on page:", buttons.length);
  buttons.forEach((btn, i) => {
    console.log(`Button ${i}:`, {
      text: btn.textContent?.trim().substring(0, 50),
      ariaLabel: btn.getAttribute('aria-label'),
      classes: btn.className,
      testId: btn.getAttribute('data-testid'),
      visible: btn.offsetParent !== null,
      disabled: btn.disabled
    });
  });
};

/**
 * Consolidated conversation state management
 * Handles tracking with proper deduplication
 */
class ConversationTracker {
  constructor(agentType) {
    this.agentType = agentType;
    this.activeConversation = null; // Only track ONE active conversation per tab
    this.lastUrl = window.location.href;
  }

  /**
   * Check conversation state and update tracking
   *
   * Simple logic:
   * - If generating: ensure task is "running"
   * - If not generating: ensure task is "completed"
   * - Create task on first detection
   *
   * @param {string} conversationId - Current conversation ID
   * @param {boolean} isGenerating - Whether generation is active NOW
   * @param {string} title - Conversation title
   */
  checkState(conversationId, isGenerating, title) {
    debugLog("Checking conversation state...");
    debugLog("  Conversation ID:", conversationId);
    debugLog("  Is generating:", isGenerating);
    debugLog("  Current active:", this.activeConversation?.conversationId);

    if (!conversationId) {
      debugLog("No conversation ID found");
      return;
    }

    // Check if this is a different conversation (user navigated)
    if (this.activeConversation && this.activeConversation.conversationId !== conversationId) {
      debugLog("Different conversation detected - resetting");
      this.activeConversation = null;
    }

    // Create task if we don't have one for this conversation
    if (!this.activeConversation) {
      // Use conversation ID as task ID for persistence across page reloads
      // This ensures follow-ups reuse the same task in the database
      const taskId = `${this.agentType}-${conversationId}`;

      debugLog("NEW CONVERSATION - creating task");
      debugLog("  Task ID:", taskId);
      debugLog("  Title:", title);

      this.activeConversation = {
        conversationId: conversationId,
        taskId: taskId,
        title: title,
        lastStatus: null,
      };
    }

    // Determine current status based on generation state
    const currentStatus = isGenerating ? "running" : "completed";

    // Only send update if status changed
    if (this.activeConversation.lastStatus !== currentStatus) {
      debugLog(`Status changed: ${this.activeConversation.lastStatus} → ${currentStatus}`);

      sendTaskUpdate(
        this.agentType,
        this.activeConversation.taskId,
        currentStatus,
        this.activeConversation.title,
        {
          url: window.location.href,
          conversation_id: conversationId,
          timestamp: Date.now(),
        }
      );

      this.activeConversation.lastStatus = currentStatus;
      console.log(`Task ${this.activeConversation.taskId}: ${currentStatus}`);
    } else {
      debugLog(`Status unchanged: ${currentStatus}`);
    }
  }

  /**
   * Mark as needs attention
   */
  markNeedsAttention(reason) {
    if (this.activeConversation && !this.activeConversation.needsAttentionReported) {
      debugLog("MARKING AS NEEDS ATTENTION:", reason);

      sendTaskUpdate(this.agentType, this.activeConversation.taskId, "needs_attention", this.activeConversation.title, {
        url: window.location.href,
        conversation_id: this.activeConversation.conversationId,
        timestamp: Date.now(),
        reason: reason,
      });

      this.activeConversation.needsAttentionReported = true;
    }
  }

  /**
   * Handle page unload
   */
  handleUnload() {
    if (this.activeConversation && this.activeConversation.isGenerating) {
      debugLog("Page unloading, marking active conversation as completed");

      sendTaskUpdate(this.agentType, this.activeConversation.taskId, "completed", this.activeConversation.title, {
        url: window.location.href,
        conversation_id: this.activeConversation.conversationId,
        timestamp: Date.now(),
        duration_ms: Date.now() - this.activeConversation.startTime,
        reason: "tab_closed",
      });
    }
  }

  /**
   * Check for URL changes
   */
  checkUrlChange() {
    const currentUrl = window.location.href;
    if (currentUrl !== this.lastUrl) {
      debugLog("URL changed from", this.lastUrl, "to", currentUrl);
      this.lastUrl = currentUrl;

      // Reset conversation tracking on URL change
      this.activeConversation = null;
      return true;
    }
    return false;
  }
}
