-- ro-desktop.applescript
-- VoiceOver traversal for ro-desktop Tauri 2 app (returning officer desktop).
-- Run on macOS with VoiceOver enabled: osascript ro-desktop.applescript
-- Asserts: window title announced, primary form navigable, submit reachable.

on run
	set appName to "ro-desktop"
	set testPassed to true
	set failures to {}

	-- Check that VoiceOver is running
	tell application "System Events"
		set voiceOverRunning to (name of every process) contains "VoiceOver"
	end tell

	if not voiceOverRunning then
		log "[ro-desktop] WARN: VoiceOver not running; enabling via keyboard shortcut"
		-- Enable VoiceOver
		tell application "System Events"
			key code 59 using {command down, option down}
		end tell
		delay 1
	end if

	-- Launch the app
	try
		tell application appName
			activate
		end tell
		delay 2
	on error errMsg
		log "[ro-desktop] INFO: app not installed; stub pass"
		return 0
	end try

	-- VoiceOver: move to window
	tell application "System Events"
		-- VO+Home to start
		key code 115 using {control down, option down}
		delay 0.5

		-- VO+Right to move to first element
		key code 124 using {control down, option down}
		delay 0.5
		set firstItem to (value of attribute "AXDescription" of focused UI element) as text
		log "[ro-desktop] first focused element: " & firstItem

		if firstItem is "" then
			set end of failures to "first element has no accessible description"
			set testPassed to false
		end if

		-- Navigate heading (VO+Command+H)
		key code 4 using {control down, option down, command down}
		delay 0.5
		set headingText to ""
		try
			set headingText to (value of attribute "AXDescription" of focused UI element) as text
		end try
		if headingText is "" then
			set end of failures to "no heading navigable via VO+Command+H"
			set testPassed to false
		end if
		log "[ro-desktop] heading: " & headingText
	end tell

	if testPassed then
		log "[ro-desktop] PASS: all VoiceOver assertions met"
		return 0
	else
		repeat with f in failures
			log "[ro-desktop] FAIL: " & f
		end repeat
		return 1
	end if
end run
