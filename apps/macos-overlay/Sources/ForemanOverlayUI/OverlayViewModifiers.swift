import SwiftUI

extension View {
    func activePanelBorder(isActive: Bool, accent: Color) -> some View {
        overlay {
            RoundedRectangle(cornerRadius: 10)
                .stroke(isActive ? accent.opacity(0.70) : .clear, lineWidth: 2)
                .allowsHitTesting(false)
        }
    }
}
