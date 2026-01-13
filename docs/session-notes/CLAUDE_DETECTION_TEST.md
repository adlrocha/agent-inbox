# Claude Detection Testing Script

Run this in the Claude.ai browser console to identify the correct selectors:

```javascript
// Test when Claude is GENERATING
function testGeneratingState() {
  console.log("=== GENERATING STATE TEST ===");

  // Check for stop button
  const stopButtons = [
    document.querySelector('[aria-label="Stop generating"]'),
    document.querySelector('button[data-testid="stop-button"]'),
    ...document.querySelectorAll('button[aria-label*="Stop"]'),
  ];
  console.log("Stop buttons found:", stopButtons.filter(b => b).length);
  stopButtons.forEach((btn, i) => {
    if (btn) console.log(`  Stop button ${i}:`, {
      visible: btn.offsetParent !== null,
      text: btn.textContent?.trim(),
      ariaLabel: btn.getAttribute('aria-label')
    });
  });

  // Check send button state
  const sendButtons = document.querySelectorAll('button[aria-label*="Send"], button[type="submit"]');
  console.log("Send buttons:", sendButtons.length);
  sendButtons.forEach((btn, i) => {
    console.log(`  Send button ${i}:`, {
      disabled: btn.disabled,
      visible: btn.offsetParent !== null,
      ariaLabel: btn.getAttribute('aria-label')
    });
  });

  // Check for streaming indicators
  const streaming = document.querySelectorAll('[data-is-streaming="true"]');
  console.log("Streaming elements:", streaming.length);

  // Check for animation/pulse
  const animated = document.querySelectorAll('.animate-pulse, .animate-spin');
  console.log("Animated elements:", animated.length);
}

// Test when Claude is IDLE (finished generating)
function testIdleState() {
  console.log("=== IDLE STATE TEST ===");

  // Check for enabled send button
  const sendButtons = document.querySelectorAll('button[aria-label*="Send"], button[type="submit"]');
  console.log("Send buttons:", sendButtons.length);
  sendButtons.forEach((btn, i) => {
    console.log(`  Send button ${i}:`, {
      disabled: btn.disabled,
      visible: btn.offsetParent !== null,
      ariaLabel: btn.getAttribute('aria-label')
    });
  });

  // Check no streaming
  const streaming = document.querySelectorAll('[data-is-streaming="true"]');
  console.log("Streaming elements:", streaming.length);

  // Check for textarea enabled
  const textareas = document.querySelectorAll('textarea');
  console.log("Textareas:", textareas.length);
  textareas.forEach((ta, i) => {
    console.log(`  Textarea ${i}:`, {
      disabled: ta.disabled,
      visible: ta.offsetParent !== null,
      placeholder: ta.placeholder
    });
  });
}

// Run both tests
console.log("Run testGeneratingState() while Claude is generating");
console.log("Run testIdleState() when Claude has finished");
```

## Expected Results

**While Generating:**
- Stop button visible: YES
- Send button disabled: YES
- Streaming elements: > 0
- Animated elements: > 0

**After Completion:**
- Stop button visible: NO
- Send button disabled: NO
- Streaming elements: 0
- Textarea enabled: YES

Use the output to update the selectors in `claude.js`.
