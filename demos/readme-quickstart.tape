Output demos/readme-quickstart.gif

Set FontSize 14
Set Width 1200
Set Height 700
Set FontFamily "Menlo"
Set LetterSpacing 0
Set LineHeight 1.2
Set Theme "TokyoNight"
Set Padding 20
Set TypingSpeed 20ms
Set PlaybackSpeed 1
Set Shell "bash"
Set WindowBar Colorful

Type@0ms "export PATH=$HOME/.cargo/bin:$PATH && cd ~/git_repositories/foreman && export PS1='$ ' && source demos/setup-readme-demo.sh"
Enter
Sleep 1500ms
Ctrl+L

Type@0ms "tmux -S $FOREMAN_DEMO_SOCKET attach-session -t operator"
Enter
Sleep 1200ms

Type "caption 81 '1/3 Open Foreman as a tmux popup'"
Enter
Sleep 900ms
Ctrl+B
Sleep 100ms
Type "a"
Sleep 2600ms
Type "q"
Sleep 1000ms

Type "caption 141 '2/3 Search narrows noisy tmux state'"
Enter
Sleep 900ms
Ctrl+B
Sleep 100ms
Type "a"
Sleep 2200ms

Type "/"
Sleep 800ms
Type "pi"
Sleep 2400ms
Escape
Sleep 900ms
Type "q"
Sleep 1000ms

Type "caption 214 '3/3 Flash labels and help explain the map'"
Enter
Sleep 900ms
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
