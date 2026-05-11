import SwiftUI

import ForemanOverlayCore

struct HelpOverlay: View {
    @ObservedObject var store: OverlayStore

    private let sections: [(String, [String])] = [
        ("Global", ["Default hotkey Cmd+Option+F toggles the overlay", "Esc closes help, cancels compose, then closes the overlay", "Cmd+, opens Settings", "Cmd+R refreshes agents", "Cmd+J shows flash jump labels", "Cmd+Shift+J jumps and focuses", "Cmd+T cycles themes", "? opens this help"]),
        ("List / Search", ["Type anywhere to search while compose is inactive", "↑/↓ moves the selected agent", "Cmd+J then a visible label jumps directly to a row", "Enter focuses the selected tmux pane", "Double-click focuses the clicked row"]),
        ("Regions", ["Tab moves List → PR → Details → Compose when a PR exists", "Rows without PR metadata skip the PR region", "Shift+Tab moves backward", "The footer chip and subtle focus ring mark the active keyboard region"]),
        ("Pull Request", ["When PR is active, Enter opens it in your browser", "Click the PR card or tab to it to make it active", "The Open PR link remains pointer-accessible"]),
        ("Details", ["When Details is active, ↑/↓ scrolls recent output", "Enter focuses the selected tmux pane", "Status source shows native, compatibility, or unknown provenance", "Preview text is selectable with the pointer"]),
        ("Compose", ["Use Compose to send text to the selected agent", "Cmd+Enter sends", "Esc cancels compose without sending"]),
        ("Status Legend", ["Working means the agent is actively running", "Needs attention means the agent is likely waiting for you", "Idle means no active work was detected", "Error means Foreman detected a broken or failed state"]),
        ("Parity Scope", ["This overlay targets quick global control, not every Foreman TUI command", "Flash jump uses Cmd+J so plain typing still searches", "Deferred TUI features include full PR actions, spawn, rename, kill, and notification controls"]),
    ]

    var body: some View {
        RoundedRectangle(cornerRadius: 18)
            .fill(.regularMaterial)
            .shadow(radius: 22)
            .overlay {
                VStack(alignment: .leading, spacing: 12) {
                    HStack {
                        Text("Foreman Help")
                            .font(.title2.weight(.semibold))
                        Spacer()
                        Text("↑/↓ scroll · Esc close")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    Divider()
                    ScrollViewReader { proxy in
                        ScrollView {
                            VStack(alignment: .leading, spacing: 16) {
                                ForEach(Array(sections.enumerated()), id: \.offset) { index, section in
                                    VStack(alignment: .leading, spacing: 6) {
                                        Text(section.0)
                                            .font(.headline)
                                        ForEach(section.1, id: \.self) { item in
                                            HStack(alignment: .top, spacing: 8) {
                                                Text("•")
                                                    .foregroundStyle(.secondary)
                                                Text(item)
                                            }
                                            .font(.callout)
                                        }
                                    }
                                    .id(index)
                                }
                            }
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 4)
                        }
                        .onAppear {
                            proxy.scrollTo(store.helpScrollOffset, anchor: .top)
                        }
                        .onChange(of: store.helpScrollOffset) { _, offset in
                            withAnimation(.easeInOut(duration: 0.12)) {
                                proxy.scrollTo(offset, anchor: .top)
                            }
                        }
                    }
                }
                .padding(20)
            }
            .frame(width: 620, height: 440)
            .padding()
    }
}
