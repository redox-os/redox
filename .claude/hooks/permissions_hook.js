#!/usr/bin/env node

/**
 * PreToolUse Hook for Claude Code
 *
 * This hook intercepts tool calls and requests user permission via MQTT
 * before allowing Claude to execute them.
 *
 * Communication Flow:
 * 1. Receives tool call details from Claude via stdin
 * 2. Reads session-to-project mapping from sessions.json
 * 3. Publishes permission request to MQTT (permissions/{projectId}/request)
 * 4. Waits for user response from frontend (permissions/{projectId}/response)
 * 5. Returns decision to Claude via stdout
 */

const mqtt = require('mqtt');
const fs = require('fs');
const path = require('path');

// Load environment variables from .env file in hooks folder (silent mode)
require('dotenv').config({ path: path.join(__dirname, '.env'), quiet: true });

// ============================================================================
// Configuration
// ============================================================================

const MQTT_CONFIG = {
  host: process.env.MQTT_HOST || 'localhost',
  port: parseInt(process.env.MQTT_PORT || '8883', 10),
  username: process.env.MQTT_USERNAME || undefined,
  password: process.env.MQTT_PASSWORD || undefined,
  clientId: `claude-hook-${Math.random().toString(16).substring(2, 10)}`,
  qos: parseInt(process.env.MQTT_QOS || '1', 10),
};

const TIMEOUT_MS = 300000; // 5 minutes default timeout
// Sessions.json is in .claude folder (same folder as hooks folder)
// Hook runs from $CLAUDE_PROJECT_DIR/.claude/hooks, so go up 1 level to .claude
const SESSION_MAPPING_FILE = path.join(__dirname, '..', 'sessions.json');
const SESSION_CONFIG_FILE = path.join(__dirname, '..', 'session-config.json');

// Tools that modify code/files - only these should trigger acceptEdits mode
const CODE_MODIFYING_TOOLS = ['Edit', 'MultiEdit', 'Write', 'NotebookEdit'];

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Logs to stderr (so it doesn't interfere with stdout JSON output)
 */
function log(message, data = null) {
  const timestamp = new Date().toISOString();
  const logMsg = data
    ? `[${timestamp}] [PermissionsHook] ${message}: ${JSON.stringify(data)}`
    : `[${timestamp}] [PermissionsHook] ${message}`;
  console.error(logMsg);
}

/**
 * Reads session-to-project mapping from sessions.json
 * Returns projectId for given sessionId
 */
function getProjectIdFromSession(sessionId) {
  try {
    if (!fs.existsSync(SESSION_MAPPING_FILE)) {
      log('Session mapping file not found', { path: SESSION_MAPPING_FILE });
      return null;
    }

    const data = fs.readFileSync(SESSION_MAPPING_FILE, 'utf8');
    const mapping = JSON.parse(data);

    if (mapping[sessionId]) {
      log('Found project ID for session', { sessionId, projectId: mapping[sessionId] });
      return mapping[sessionId];
    }

    log('Session ID not found in mapping', { sessionId });
    return null;

  } catch (error) {
    log('Error reading session mapping', { error: error.message });
    return null;
  }
}

/**
 * Reads permission mode for given session from session-config.json
 * Returns permission mode ('default', 'acceptEdits', 'plan', 'bypassPermissions')
 */
function getPermissionModeForSession(sessionId) {
  try {
    if (!fs.existsSync(SESSION_CONFIG_FILE)) {
      log('Session config file not found, using default mode', { path: SESSION_CONFIG_FILE });
      return 'default';
    }

    const data = fs.readFileSync(SESSION_CONFIG_FILE, 'utf8');
    const config = JSON.parse(data);

    if (config[sessionId] && config[sessionId].permissionMode) {
      const mode = config[sessionId].permissionMode;
      log('Found permission mode for session', { sessionId, permissionMode: mode });
      return mode;
    }

    log('Session ID not found in config, using default mode', { sessionId });
    return 'default';

  } catch (error) {
    log('Error reading session config, using default mode', { error: error.message });
    return 'default';
  }
}

/**
 * Reads the current working directory for a session from session-config.json
 * Returns the cwd or null if not found
 */
function getCwdForSession(sessionId) {
  try {
    if (!fs.existsSync(SESSION_CONFIG_FILE)) {
      log('Session config file not found, no cwd available', { path: SESSION_CONFIG_FILE });
      return null;
    }

    const data = fs.readFileSync(SESSION_CONFIG_FILE, 'utf8');
    const config = JSON.parse(data);

    if (config[sessionId] && config[sessionId].cwd) {
      const cwd = config[sessionId].cwd;
      log('Found cwd for session', { sessionId, cwd });
      return cwd;
    }

    log('No cwd found for session', { sessionId });
    return null;

  } catch (error) {
    log('Error reading cwd from session config', { error: error.message });
    return null;
  }
}

/**
 * Checks if a file path is outside the project's current working directory
 * Returns true if the file is outside the project directory
 */
function isFileOutsideProjectDirectory(filePath, projectCwd) {
  try {
    if (!filePath || !projectCwd) {
      return false;
    }

    // Resolve both paths to absolute paths
    const absoluteFilePath = path.resolve(filePath);
    const absoluteProjectCwd = path.resolve(projectCwd);

    // Check if the file path starts with the project cwd
    const isInside = absoluteFilePath.startsWith(absoluteProjectCwd + path.sep) ||
                     absoluteFilePath === absoluteProjectCwd;

    log('Checking if file is outside project directory', {
      filePath: absoluteFilePath,
      projectCwd: absoluteProjectCwd,
      isOutside: !isInside
    });

    return !isInside;

  } catch (error) {
    log('Error checking file path', { error: error.message });
    // On error, be conservative and ask for permission
    return true;
  }
}

/**
 * Reads JSON input from stdin
 */
function readStdin() {
  return new Promise((resolve, reject) => {
    let data = '';

    process.stdin.setEncoding('utf8');

    process.stdin.on('data', (chunk) => {
      data += chunk;
    });

    process.stdin.on('end', () => {
      try {
        const parsed = JSON.parse(data);
        resolve(parsed);
      } catch (error) {
        reject(new Error(`Failed to parse stdin JSON: ${error.message}`));
      }
    });

    process.stdin.on('error', (error) => {
      reject(new Error(`Error reading stdin: ${error.message}`));
    });
  });
}

/**
 * Outputs the decision to stdout in the format Claude expects
 */
function outputDecision(decision, reason = '') {
  const output = {
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      permissionDecision: decision,
      permissionDecisionReason: reason
    }
  };

  console.log(JSON.stringify(output));
}

/**
 * Generates a permission string for Claude Code's settings.local.json format
 * Format: "ToolName(parameter)" - e.g., "Bash(npm install)", "Edit(/path/to/file.ts)"
 */
function generatePermissionString(toolName, toolInput) {
  try {
    if (!toolInput) {
      return toolName;
    }

    switch (toolName) {
      case 'Bash':
        return toolInput.command ? `Bash(${toolInput.command})` : 'Bash';
      case 'Edit':
        return toolInput.file_path ? `Edit(${toolInput.file_path})` : 'Edit';
      case 'Write':
        return toolInput.file_path ? `Write(${toolInput.file_path})` : 'Write';
      case 'Read':
        return toolInput.file_path ? `Read(${toolInput.file_path})` : 'Read';
      case 'WebFetch':
        return toolInput.url ? `WebFetch(${toolInput.url})` : 'WebFetch';
      case 'WebSearch':
        return toolInput.query ? `WebSearch(${toolInput.query})` : 'WebSearch';
      case 'NotebookEdit':
        return toolInput.notebook_path ? `NotebookEdit(${toolInput.notebook_path})` : 'NotebookEdit';
      case 'BashOutput':
        return 'BashOutput';
      default:
        return toolName;
    }
  } catch (error) {
    log('Error generating permission string', { error: error.message });
    return toolName;
  }
}

/**
 * Checks if a tool+command is in the allowed list (settings.local.json)
 * Returns true if permission is granted via allowed list
 */
function checkAllowedTools(toolName, toolInput) {
  try {
    const settingsPath = path.join(__dirname, '..', 'settings.local.json');

    if (!fs.existsSync(settingsPath)) {
      return false;
    }

    const settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));

    if (!settings.permissions || !settings.permissions.allow) {
      return false;
    }

    // Generate the same permission string format used when adding
    const permissionString = generatePermissionString(toolName, toolInput);

    // Check if this exact permission exists
    const isAllowed = settings.permissions.allow.includes(permissionString);

    if (isAllowed) {
      log('Tool+command found in allowed list', { permissionString });
      return true;
    }

    return false;
  } catch (error) {
    log('Error checking allowed tools', { error: error.message });
    return false;
  }
}

// ============================================================================
// MQTT Communication
// ============================================================================

/**
 * Connects to MQTT broker and waits for permission response
 */
async function requestPermission(hookInput, projectId) {
  const { tool_name, tool_input, session_id, cwd } = hookInput;

  log('Processing permission request', { tool_name, projectId, session_id });

  return new Promise((resolve, reject) => {
    // Use WebSocket protocol for MQTT connection (embedded broker uses ws://)
    const connectUrl = `ws://${MQTT_CONFIG.host}:${MQTT_CONFIG.port}`;

    const connectOptions = {
      clientId: MQTT_CONFIG.clientId,
      clean: true,
      reconnectPeriod: 0, // Don't auto-reconnect (we need fast failure)
      connectTimeout: 10000,
    };

    // Add credentials if provided
    if (MQTT_CONFIG.username && MQTT_CONFIG.password) {
      connectOptions.username = MQTT_CONFIG.username;
      connectOptions.password = MQTT_CONFIG.password;
    }

    log('Connecting to MQTT broker', { host: MQTT_CONFIG.host, port: MQTT_CONFIG.port });

    const client = mqtt.connect(connectUrl, connectOptions);
    let resolved = false;
    let timeoutHandle = null;
    let ackTimeoutHandle = null;
    let ackReceived = false;

    // Topics for communication
    const requestTopic = `permissions/${projectId}/request`;
    const responseTopic = `permissions/${projectId}/response`;
    const ackTopic = `permissions/${projectId}/ack`;

    /**
     * Cleanup and resolve/reject helper
     */
    function cleanup(callback) {
      if (resolved) return;
      resolved = true;

      if (timeoutHandle) {
        clearTimeout(timeoutHandle);
      }

      if (ackTimeoutHandle) {
        clearTimeout(ackTimeoutHandle);
      }

      // Clean disconnect
      client.end(true, {}, () => {
        callback();
      });
    }

    // Set timeout for user response
    timeoutHandle = setTimeout(() => {
      log('Permission request timed out', { timeout: TIMEOUT_MS });
      cleanup(() => {
        resolve({
          decision: 'deny',
          reason: `Permission request timed out after ${TIMEOUT_MS / 1000} seconds. No response from user.`
        });
      });
    }, TIMEOUT_MS);

    // Handle connection
    client.on('connect', () => {
      log('Connected to MQTT broker');

      // Subscribe to ACK topic first
      client.subscribe(ackTopic, { qos: MQTT_CONFIG.qos }, (err) => {
        if (err) {
          log('Failed to subscribe to ACK topic', { error: err.message });
          cleanup(() => {
            reject(new Error(`Failed to subscribe to ACK: ${err.message}`));
          });
          return;
        }

        log('Subscribed to ACK topic', { topic: ackTopic });

        // Subscribe to response topic
        client.subscribe(responseTopic, { qos: MQTT_CONFIG.qos }, (err) => {
          if (err) {
            log('Failed to subscribe to response topic', { error: err.message });
            cleanup(() => {
              reject(new Error(`Failed to subscribe: ${err.message}`));
            });
            return;
          }

          log('Subscribed to response topic', { topic: responseTopic });

          // Publish permission request
          const permissionRequest = {
            tool_name,
            tool_input,
            session_id,
            cwd,
            timestamp: Date.now(),
            project_id: projectId,
          };

          const payload = JSON.stringify(permissionRequest);

          client.publish(requestTopic, payload, { qos: MQTT_CONFIG.qos, retain: false }, (err) => {
            if (err) {
              log('Failed to publish permission request', { error: err.message });
              cleanup(() => {
                reject(new Error(`Failed to publish: ${err.message}`));
              });
              return;
            }

            log('Published permission request', { topic: requestTopic });

            // Send notification to all devices
            const notificationTopic = `notifications/${projectId}/permission`;
            const notificationPayload = JSON.stringify({
              type: 'permission_request',
              tool_name,
              tool_input,
              timestamp: Date.now(),
              project_id: projectId,
            });

            client.publish(notificationTopic, notificationPayload, { qos: MQTT_CONFIG.qos }, (notifErr) => {
              if (notifErr) {
                log('Failed to publish notification', { error: notifErr.message });
                // Don't fail the whole request if notification fails
              } else {
                log('Published permission notification', { topic: notificationTopic });
              }
            });

            // Set ACK timeout (2 seconds)
            ackTimeoutHandle = setTimeout(() => {
              if (!ackReceived) {
                log('ACK not received within 2 seconds - user likely not on mobile app');
                cleanup(() => {
                  // Exit without response - Claude will continue without permission
                  process.exit(0);
                });
              }
            }, 5000);
          });
        });
      });
    });

    // Handle incoming messages (ACK and response from frontend)
    client.on('message', (topic, payload) => {
      // Handle ACK
      if (topic === ackTopic) {
        try {
          const ack = JSON.parse(payload.toString());
          log('Received ACK from mobile app', ack);
          ackReceived = true;

          // Clear ACK timeout since we received it
          if (ackTimeoutHandle) {
            clearTimeout(ackTimeoutHandle);
            ackTimeoutHandle = null;
          }
        } catch (error) {
          log('Failed to parse ACK', { error: error.message });
        }
        return;
      }

      // Handle response
      if (topic !== responseTopic) return;

      try {
        const response = JSON.parse(payload.toString());
        log('Received permission response', response);

        // Validate response format
        if (!response.decision || !['allow', 'deny'].includes(response.decision)) {
          log('Invalid response format', response);
          cleanup(() => {
            resolve({
              decision: 'deny',
              reason: 'Invalid response format from frontend'
            });
          });
          return;
        }

        // If permission mode is updated in response, save it
        if (response.permissionMode) {
          log('Updating permission mode from response', {
            sessionId: session_id,
            newMode: response.permissionMode
          });

          try {
            const configPath = path.join(__dirname, '..', 'session-config.json');
            let config = {};

            if (fs.existsSync(configPath)) {
              config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
            }

            config[session_id] = { permissionMode: response.permissionMode };
            fs.writeFileSync(configPath, JSON.stringify(config, null, 2), 'utf8');

            log('Permission mode updated successfully', { sessionId: session_id });
          } catch (error) {
            log('Failed to update permission mode', { error: error.message });
          }
        }

        // If user clicked "don't ask again", add to allowed tools in settings.local.json
        if (response.addToAllowedTools) {
          // Generate permission string from the tool_name and tool_input we received earlier
          const permissionString = generatePermissionString(tool_name, tool_input);

          log('Adding permission to settings.local.json', {
            toolName: tool_name,
            permissionString
          });

          try {
            const settingsPath = path.join(__dirname, '..', 'settings.local.json');
            let settings = {};

            // Read existing settings
            if (fs.existsSync(settingsPath)) {
              settings = JSON.parse(fs.readFileSync(settingsPath, 'utf8'));
            }

            // Initialize permissions structure if needed
            if (!settings.permissions) {
              settings.permissions = {};
            }
            if (!settings.permissions.allow) {
              settings.permissions.allow = [];
            }

            // Add permission string if not already present
            if (!settings.permissions.allow.includes(permissionString)) {
              settings.permissions.allow.push(permissionString);

              // Write back atomically
              const tempPath = `${settingsPath}.tmp`;
              fs.writeFileSync(tempPath, JSON.stringify(settings, null, 2), 'utf8');
              fs.renameSync(tempPath, settingsPath);

              log('Added permission to settings.local.json', {
                permissionString,
                totalAllowed: settings.permissions.allow.length
              });
            } else {
              log('Permission already exists in settings.local.json', { permissionString });
            }
          } catch (error) {
            log('Failed to update settings.local.json', { error: error.message });
          }
        }

        // Valid response received
        cleanup(() => {
          resolve({
            decision: response.decision,
            reason: response.reason || `User ${response.decision}ed the tool use`
          });
        });

      } catch (error) {
        log('Failed to parse response', { error: error.message });
        cleanup(() => {
          resolve({
            decision: 'deny',
            reason: 'Failed to parse permission response'
          });
        });
      }
    });

    // Handle connection errors
    client.on('error', (error) => {
      log('MQTT connection error', { error: error.message });
      cleanup(() => {
        reject(new Error(`MQTT error: ${error.message}`));
      });
    });

    // Handle unexpected disconnection
    client.on('close', () => {
      if (!resolved) {
        log('MQTT connection closed unexpectedly');
        cleanup(() => {
          reject(new Error('MQTT connection closed before receiving response'));
        });
      }
    });
  });
}

// ============================================================================
// Main Execution
// ============================================================================

async function main() {
  try {
    log('Hook started');

    // Read hook input from stdin
    const hookInput = await readStdin();
    log('Received hook input', hookInput);

    // Validate required fields
    if (!hookInput.tool_name || !hookInput.session_id || !hookInput.cwd) {
      throw new Error('Missing required fields in hook input');
    }

    // Get project ID from session mapping file
    let projectId = getProjectIdFromSession(hookInput.session_id);

    // If no project ID found, we can't route the permission request via MQTT
    // This could happen if the session was started from CLI without backend
    // In this case, exit without blocking (allow the operation)
    if (!projectId) {
      log('Session not found in sessions.json - allowing operation without permission check');
      process.exit(0);
    }

    log('Resolved project ID', { projectId });

    // Get permission mode for this session
    const permissionMode = getPermissionModeForSession(hookInput.session_id);
    const toolName = hookInput.tool_name;

    log('Permission mode check', { sessionId: hookInput.session_id, permissionMode, toolName });

    // Auto-approval logic based on permission mode
    if (permissionMode === 'bypassPermissions') {
      log('Auto-approving (bypass mode)');
      outputDecision('allow', 'User has approved the tool use (bypass mode)');
      process.exit(0);
    }

    if (permissionMode === 'acceptEdits') {
      // Only auto-approve actual code-modifying tools
      if (CODE_MODIFYING_TOOLS.includes(toolName)) {
        log('Auto-approving (acceptEdits mode)', { toolName });
        outputDecision('allow', `User has approved code-modifying tools`);
        process.exit(0);
      }
      log('Tool not auto-approved in acceptEdits mode, asking user', { toolName });
    }

    // Plan mode - will handle separately with special UI
    // For now, ask user for all tools in plan mode
    if (permissionMode === 'plan') {
      log('Plan mode detected - will ask user with special UI');
      // TODO: Plan mode will need special handling in frontend
      // For now, ask user normally
    }

    // Special handling for Read tool - check if file is outside project directory
    if (toolName === 'Read') {
      const projectCwd = getCwdForSession(hookInput.session_id);
      const filePath = hookInput.tool_input?.file_path;

      if (projectCwd && filePath) {
        const isOutside = isFileOutsideProjectDirectory(filePath, projectCwd);

        if (!isOutside) {
          // File is inside project directory, auto-approve
          log('Auto-approving Read (file inside project directory)', { filePath, projectCwd });
          outputDecision('allow', 'File is within project directory');
          process.exit(0);
        } else {
          // File is outside project directory, will ask user below
          log('Read operation outside project directory, will ask user', { filePath, projectCwd });
        }
      }
    }

    // Check if this specific tool+command is in allowed list
    if (checkAllowedTools(toolName, hookInput.tool_input)) {
      log('Auto-approving (found in allowed list)');
      outputDecision('allow', `User has allowed this tool use previously for auto-approval`);
      process.exit(0);
    }

    // Default mode or plan mode - ask user via MQTT
    log('Requesting permission via MQTT', { mode: permissionMode });
    const result = await requestPermission(hookInput, projectId);
    log('Permission decision made', result);

    // Output decision to Claude
    outputDecision(result.decision, result.reason);

    // Exit with appropriate code
    process.exit(0);

  } catch (error) {
    log('Hook execution failed', { error: error.message, stack: error.stack });

    // On error, deny the tool use
    outputDecision('deny', `Hook error: ${error.message}`);
    process.exit(0); // Exit 0 even on error, but with deny decision
  }
}

// Handle process signals
process.on('SIGINT', () => {
  log('Received SIGINT, exiting');
  outputDecision('deny', 'Hook interrupted by signal');
  process.exit(0);
});

process.on('SIGTERM', () => {
  log('Received SIGTERM, exiting');
  outputDecision('deny', 'Hook terminated by signal');
  process.exit(0);
});

// Run the hook
main();
