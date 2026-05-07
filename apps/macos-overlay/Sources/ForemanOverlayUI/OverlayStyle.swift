import SwiftUI

func prStatusColor(_ status: String) -> Color {
    switch status {
    case "draft": return .orange
    case "closed": return .secondary
    case "merged": return .purple
    default: return .green
    }
}

func statusColor(_ status: String) -> Color {
    switch status {
    case "error": return .red
    case "needs-attention": return .orange
    case "working": return .blue
    case "idle": return .secondary
    default: return .gray
    }
}
