Output demos/readme-quickstart.gif

Set FontSize 14
Set Width 1200
Set Height 700
Set FontFamily "Menlo"
Set LetterSpacing 0
Set LineHeight 1.2
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 20ms
Set PlaybackSpeed 1
Set Shell "bash"
Set WindowBar Colorful

Type@0ms "export PATH=$HOME/.cargo/bin:$PATH && cd ~/git_repositories/foreman && export PS1='$ ' && export FOREMAN_CONFIG_HOME=/tmp/foreman-readme-demo/config && export XDG_STATE_HOME=/tmp/foreman-readme-demo/state && export FOREMAN_LOG_DIR=/tmp/foreman-readme-demo/logs && rm -rf /tmp/foreman-readme-demo && mkdir -p /tmp/foreman-readme-demo"
Enter
Sleep 1s
Ctrl+L

Type "foreman --help | sed -n '1,32p'"
Enter
Sleep 3s

Type "foreman --config-show | sed -n '1,28p'"
Enter
Sleep 3s

Type "foreman --doctor --doctor-fix --doctor-dry-run"
Enter
Sleep 5s
