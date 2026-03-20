import SwiftUI

struct HelpView: View {
    var body: some View {
        List {
            Section("Getting Started") {
                HelpRow(
                    icon: "folder",
                    title: "Browse your drive",
                    detail: "Open folders from My Drive to find files, then tap a file to preview it."
                )
                HelpRow(
                    icon: "arrow.up.circle",
                    title: "Upload files",
                    detail: "Use the menu in the top-right corner of My Drive and choose Upload."
                )
                HelpRow(
                    icon: "folder.badge.plus",
                    title: "Create folders",
                    detail: "Choose New Folder from the same menu to organize your files."
                )
            }

            Section("Previews") {
                HelpRow(
                    icon: "doc.text.magnifyingglass",
                    title: "Open previews",
                    detail: "Tap a file or use the Preview action from the file menu."
                )
                HelpRow(
                    icon: "sparkles.rectangle.stack",
                    title: "Neutrino files",
                    detail: "Native Docs, Sheets, and Slides open in the in-app preview instead of generic Quick Look."
                )
                HelpRow(
                    icon: "plus.magnifyingglass",
                    title: "Zoom controls",
                    detail: "Images, PDFs, text files, CSVs, and supported documents can be zoomed with the on-screen controls."
                )
            }

            Section("Offline Access") {
                HelpRow(
                    icon: "arrow.down.circle",
                    title: "Make files available offline",
                    detail: "Long-press a file and choose Download for Offline to keep a local copy."
                )
                HelpRow(
                    icon: "wifi.slash",
                    title: "View offline files",
                    detail: "Use the filter menu and switch to Available Offline to see files saved on the device."
                )
            }

            Section("Security") {
                HelpRow(
                    icon: "faceid",
                    title: "Face ID or Touch ID",
                    detail: "Open Settings to enable biometric unlock when your device supports it."
                )
                HelpRow(
                    icon: "server.rack",
                    title: "Server connection",
                    detail: "If the app cannot connect, open Settings and confirm the server URL is correct."
                )
            }

            Section("Troubleshooting") {
                HelpRow(
                    icon: "arrow.clockwise",
                    title: "Refresh your drive",
                    detail: "Use Refresh from the top-right menu if file changes are not showing up yet."
                )
                HelpRow(
                    icon: "exclamationmark.triangle",
                    title: "Preview unavailable",
                    detail: "If a file will not preview, try downloading it for offline use and opening it again."
                )
            }
        }
        .navigationTitle("Help")
    }
}

private struct HelpRow: View {
    let icon: String
    let title: String
    let detail: String

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: icon)
                .font(.headline)
                .foregroundColor(.accentColor)
                .frame(width: 24)

            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.headline)
                Text(detail)
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}
