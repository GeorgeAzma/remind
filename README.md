# Remind
Easy to use Reminder CLI

### Examples
``` bash
remind 1d "code tomorrow"
remind minute "egg ready" repeat 4
remind remove "egg" # fuzzy remove
remind 12:30:15 feb 28 2029
remind monday fri "study"
remind weekly work "go to work" # 5 days a week, at current time
remind weekend "rest" rep 8
remind skip 2 "rest" # skip 2 weekends cause boss sucks
remind daily 11am workout
remind undo
remind list
remind clear
remind help
```

### Features
- command aliases (lot of them)
- simplicity
- windows notification popups
- simple reminders.txt data file
- minimal cpu usage background task
- undo + history

### How To Run (Windows)
compile it via cargo and add executable folder to environment path
then create remind.vbs in startup folder (WIN+R and type shell:startup)
``` vb
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run """remind.exe""", 0, False
```
this creates background daemon on every log in automatically
which is responsible for actually pushing the notifications

then just run remind commands anywhere
``` bash
remind day "code"
```

### Notes
- reminders are saved in `C:/User/AppData/Local/Remind/reminders.txt`
- technically this should work on linux, but it's untested