import SwiftUI

import ForemanOverlayCore

struct PreviewOutputView: View {
    let entry: AgentEntry
    @ObservedObject var store: OverlayStore

    var body: some View {
        let lines = (entry.preview.isEmpty ? "No recent preview captured." : entry.preview)
            .split(separator: "\n", omittingEmptySubsequences: false)
            .map(String.init)
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 2) {
                    ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                        Text(line.isEmpty ? " " : line)
                            .id(index)
                            .font(.system(size: 12, design: .monospaced))
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
                .textSelection(.enabled)
                .padding(10)
            }
            .scrollDisabled(store.isHelpVisible)
            .background(.black.opacity(0.08), in: RoundedRectangle(cornerRadius: 10))
            .onTapGesture { store.activateRegion(.detail) }
            .onAppear {
                proxy.scrollTo(store.previewScrollOffset, anchor: .top)
            }
            .onChange(of: store.previewScrollOffset) { _, offset in
                withAnimation(.easeInOut(duration: 0.12)) {
                    proxy.scrollTo(offset, anchor: .top)
                }
            }
        }
    }
}
