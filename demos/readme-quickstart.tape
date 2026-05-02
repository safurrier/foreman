Output demos/readme-quickstart.gif

Set FontSize 20
Set Width 1400
Set Height 820
Set FontFamily "Menlo"
Set LetterSpacing 0
Set LineHeight 1.2
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 20ms
Set PlaybackSpeed 1
Set Shell "bash"
Set WindowBar Colorful

Type@0ms "unset NO_COLOR; export PATH=$HOME/.cargo/bin:$PATH COLORTERM=truecolor CLICOLOR_FORCE=1 PS1='$ '; source demos/setup-readme-demo.sh >/dev/null && tmux -S $FOREMAN_DEMO_SOCKET attach-session -t operator"
Enter
Sleep 2200ms

Type "caption 81 '1/3 Open Foreman as a tmux popup'"
Enter
Sleep 900ms
Ctrl+B
Sleep 100ms
Type "a"
Sleep 3600ms

Type "f"
Sleep 2200ms

Ctrl+B
Sleep 100ms
Type "a"
Sleep 2200ms

Type "/"
Sleep 800ms
Type "pi"
Sleep 2400ms
Enter
Sleep 2200ms

Ctrl+B
Sleep 100ms
Type "a"
Sleep 2200ms
Type "s"
Sleep 2400ms
Escape
Sleep 900ms

Type "?"
Sleep 2600ms
Escape
Sleep 700ms
Type "q"
