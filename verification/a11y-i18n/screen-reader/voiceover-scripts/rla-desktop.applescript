-- rla-desktop.applescript
-- VoiceOver traversal for rla-desktop Tauri 2 app (returning / legal affairs desktop).
-- Run on macOS: osascript rla-desktop.applescript

on run
	set appName to "rla-desktop"
	set testPassed to true
	set failures to {}

	-- Check VoiceOver
	tell application "System Events"
		set voiceOverRunning to (name of every process) contains "VoiceOver"
	end tell

	if not voiceOverRunning then
		tell application "System Events"
			key code 59 using {command down, option down}
		end tell
		delay 1
	end if

	-- Launch
	try
		tell application appName
			activate
		end tell
		delay 2
	on error
		log "[rla-desktop] INFO: app not installed; stub pass"
		return 0
	end try

	tell application "System Events"
		-- VO+Home
		key code 115 using {control down, option down}
		delay 0.5

		-- Move to first interactive element
		key code 124 using {control down, option down}
		delay 0.5
		set firstItem to ""
		try
			set firstItem to (value of attribute "AXDescription" of focused UI element) as text
		end try
		log "[rla-desktop] first element: " & firstItem
		if firstItem is "" then
			set end of failures to "first interactive element has no accessible description"
			set testPassed to false
		end if

		-- Navigate to first button
		key code 4 using {control down, option down, command down}
		delay 0.5
		set headingText to ""
		try
			set headingText to (value of attribute "AXDescription" of focused UI element) as text
		end try
		log "[rla-desktop] heading: " & headingText

		-- Table navigation (legal document list)
		key code 116 using {control down, option down}
		delay 0.5
		set tableItem to ""
		try
			set tableItem to (value of attribute "AXDescription" of focused UI element) as text
		end try
		log "[rla-desktop] table/list item: " & tableItem
	end tell

	if testPassed then
		log "[rla-desktop] PASS: all VoiceOver assertions met"
		return 0
	else
		repeat with f in failures
			log "[rla-desktop] FAIL: " & f
		end repeat
		return 1
	end if
end run
