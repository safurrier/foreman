Output demos/readme-quickstart.gif

Set FontSize 14
Set Width 1200
Set Height 700
Set FontFamily "Menlo"
Set LetterSpacing 0
Set LineHeight 1.2
Set Theme "Cobalt Neon"
Set Padding 20
Set TypingSpeed 20ms
Set PlaybackSpeed 1
Set Shell "bash"
Set WindowBar Colorful

Type@0ms "export PATH=$HOME/.cargo/bin:$PATH && cd ~/git_repositories/foreman && export PS1='$ ' && source demos/setup-readme-demo.sh"
Enter
Sleep 1500ms
Ctrl+L

Type@0ms "target/debug/foreman --no-notify --tmux-socket $FOREMAN_DEMO_SOCKET --log-dir $FOREMAN_DEMO_LOG_DIR --config-file $FOREMAN_DEMO_CONFIG --claude-native-dir $FOREMAN_DEMO_CLAUDE_NATIVE_DIR --codex-native-dir $FOREMAN_DEMO_CODEX_NATIVE_DIR --pi-native-dir $FOREMAN_DEMO_PI_NATIVE_DIR --poll-interval-ms 250 --capture-lines 40"
Enter
Sleep 2600ms

Type "/"
Sleep 800ms
Type "pi-dots"
Sleep 2400ms
Escape
Sleep 900ms

Type "s"
Sleep 2400ms
Escape
Sleep 900ms

Type "h"
Sleep 1000ms
Type "h"
Sleep 1000ms
Type "q"
