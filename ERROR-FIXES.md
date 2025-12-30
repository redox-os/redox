# Fixing Redox OS Serial Console Errors

## The Two Main Errors

### 1. Ion Prompt Expansion Error
```
ion: prompt expansion failed: pipeline execution error:
error reading stdout of child: No such device (os error 19)
```

**Cause:** Ion shell's default prompt tries to execute commands that access devices (like `/dev/pts`) which don't exist in serial console mode.

**Solution:** Use a simple static prompt without expansions.

### 2. Localization System Error
```
Could not init the localization system: Bundle error: Localizer already initialized
```

**Cause:** The localization system tries to initialize multiple times, and some locale files may not be properly set up for serial console.

**Solution:** Disable localization by setting `LC_ALL=C` and `LANG=C`.

## Quick Fix (Current Session)

### Method 1: Manual Fix in VM
```bash
# Connect to VM
redox-vm shell test

# At the prompt (even with errors), type:
export PROMPT="redox> "
export LC_ALL="C"
export LANG="C"

# Errors disappear!
```

### Method 2: Automated Quick Fix
```bash
# From host (while VM is running)
./quick-fix.sh test

# Reconnect to see fixes
redox-vm shell test
```

## Permanent Fix

### Method 1: Create Ion Config Manually

Inside the VM:
```bash
# Create config directory
mkdir -p /home/user/.config/ion

# Create configuration file
cat > /home/user/.config/ion/ionrc << 'EOF'
# Ion Shell Configuration - Serial Console Optimized

# Fix prompt expansion errors
export PROMPT = "redox> "

# Fix localization errors
export LC_ALL = "C"
export LANG = "C"

# Terminal settings
export TERM = "vt100"

# Welcome message
echo ""
echo "Redox OS - Serial Console Mode"
echo "Type 'help' for available commands"
echo ""
EOF
```

### Method 2: Automated Complete Fix
```bash
# From host - creates persistent configuration
./apply-all-fixes.sh test

# That's it! Future logins will be error-free
```

## Verification

After applying fixes, reconnect:
```bash
redox-vm shell test
```

You should see:
```
redox login: user
Welcome to Redox OS!

Redox OS - Serial Console Mode
Type 'help' for available commands

redox>                    # ← Clean prompt!
redox> ls                 # ← No errors!
redox> pwd                # ← Works perfectly!
```

## Script Reference

| Script | Purpose | Usage |
|--------|---------|-------|
| `quick-fix.sh` | Fix current session | `./quick-fix.sh test` |
| `apply-all-fixes.sh` | Create permanent config | `./apply-all-fixes.sh test` |
| `create-ionrc.sh` | Alternative config creator | `./create-ionrc.sh test` |
| `fix-ion-prompt.sh` | Fix prompt only | `./fix-ion-prompt.sh test` |

## Understanding the Configuration

### Ion Shell Config Location
```
/home/user/.config/ion/ionrc
```

This file is sourced automatically when Ion starts.

### What Each Setting Does

```bash
# Disable command expansions in prompt
export PROMPT = "redox> "

# Disable locale system (prevents localization errors)
export LC_ALL = "C"
export LANG = "C"

# Set terminal type for compatibility
export TERM = "vt100"
```

## Advanced Customization

### Custom Prompts

Simple prompts that work well:
```bash
export PROMPT = "$ "                    # Minimal
export PROMPT = "redox> "               # Named
export PROMPT = "[\$PWD]$ "            # With directory
export PROMPT = "[\$USER@redox]$ "     # With username
```

Avoid prompts with command substitutions:
```bash
# DON'T use these in serial console:
export PROMPT = "$(git branch)> "      # ❌ Tries to access devices
export PROMPT = "$(jobs)> "            # ❌ Device access
```

### Additional Settings

```bash
# Add to ionrc for more improvements:

# Editor
export EDITOR = "nano"

# Pager
export PAGER = "less"

# History
export HISTFILE = "/home/user/.local/share/ion/history"

# Aliases
alias ll = "ls -la"
alias .. = "cd .."
alias ... = "cd ../.."

# Path additions
export PATH = "$PATH:/home/user/bin"
```

## Troubleshooting

### Errors Still Appear After Fix

1. **Check if ionrc exists:**
   ```bash
   ls -la /home/user/.config/ion/ionrc
   ```

2. **Verify ionrc content:**
   ```bash
   cat /home/user/.config/ion/ionrc
   ```

3. **Reapply fixes:**
   ```bash
   ./apply-all-fixes.sh test
   ```

4. **Manual verification:**
   ```bash
   # Inside VM
   echo $PROMPT
   echo $LC_ALL
   echo $LANG
   ```

### Config Not Loading

If the config doesn't seem to load:

1. **Check file permissions:**
   ```bash
   chmod 644 /home/user/.config/ion/ionrc
   ```

2. **Source it manually:**
   ```bash
   source /home/user/.config/ion/ionrc
   ```

3. **Check for syntax errors:**
   ```bash
   cat /home/user/.config/ion/ionrc
   ```

### Different User

If you login as a different user:
```bash
# Create config for that user
mkdir -p /home/otheruser/.config/ion
cp /home/user/.config/ion/ionrc /home/otheruser/.config/ion/
```

## Testing the Fixes

### Test Script
```bash
# Connect and test
redox-vm shell test

# Try various commands
ls
pwd
echo $PROMPT
echo $LC_ALL
cat /home/user/.config/ion/ionrc

# Should all work without errors!
```

### What Success Looks Like

**Before:**
```
>>> ls
ion: prompt expansion failed: pipeline execution error: error reading stdout of child: No such device (os error 19)
Could not init the localization system: Bundle error: Localizer already initialized
>>>
```

**After:**
```
redox> ls
file1.txt  file2.txt  directory/
redox> pwd
/home/user
redox>
```

## Summary

1. **Quick fix** (temporary): `./quick-fix.sh test`
2. **Permanent fix** (recommended): `./apply-all-fixes.sh test`
3. **Manual fix**: Create `/home/user/.config/ion/ionrc` with exports
4. **Verify**: Reconnect and test - no more errors!

Both errors are now **completely fixed** with proper Ion shell configuration! 🎉
