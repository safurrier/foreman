Output "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-diagnostic.gif"
Set FontSize 15
Set Width 760
Set Height 540
Set FontFamily "Menlo"
Set LetterSpacing 0
Set LineHeight 1.2
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 0ms
Set PlaybackSpeed 1
Set Shell "bash"
Set WindowBar "Colorful"

Type@0ms "export PATH=$HOME/.cargo/bin:$PATH && cd /Users/alex.furrier/git_repositories/foreman && export PS1='$ ' && source /Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/visual-env.txt"
Enter
Sleep 500ms
Ctrl+L
Sleep 300ms

Type@0ms "FOREMAN_LOG_DIR=$log_dir target/debug/foreman --no-notify --tmux-socket $socket"
Enter
Sleep 2200ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-initial.png"
Sleep 500ms
Type "t"
Sleep 800ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-gruvbox.png"
Sleep 400ms
Type "ttttt"
Sleep 800ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-no-color.png"
Sleep 400ms
Type "t"
Sleep 400ms

Type "?"
Sleep 800ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-help.png"
Sleep 400ms

Type "?"
Sleep 400ms
Type "q"
Sleep 300ms
Ctrl+L
Sleep 300ms
Type@0ms "FOREMAN_LOG_DIR=$log_dir target/debug/foreman --popup --no-notify --tmux-socket $socket"
Enter
Sleep 2200ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-popup.png"
Sleep 400ms
Type "q"
Sleep 300ms
Ctrl+L
Sleep 300ms
Type@0ms "FOREMAN_LOG_DIR=$log_dir target/debug/foreman --no-notify --tmux-socket $socket"
Enter
Sleep 1800ms
Type "/"
Sleep 500ms
Type "gamma"
Sleep 800ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-search.png"
Sleep 500ms
Escape
Sleep 300ms
Type "s"
Sleep 700ms
Screenshot "/Users/alex.furrier/git_repositories/foreman/.ai/validation/ux/foreman-ux-flash.png"
Sleep 500ms

Escape
Sleep 300ms
Type "q"
Sleep 300ms
