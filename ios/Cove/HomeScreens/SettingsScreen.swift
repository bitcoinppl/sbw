import LocalAuthentication
import SwiftUI

struct SettingsScreen: View {
    @Environment(AppManager.self) private var app
    @Environment(\.dismiss) private var dismiss

    @State private var notificationFrequency = 1
    @State private var networkChanged = false
    @State private var showConfirmationAlert = false

    let themes = allColorSchemes()

    private func canUseBiometrics() -> Bool {
        let context = LAContext()
        var error: NSError?
        return context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error)
    }

    var useAuth: Binding<Bool> {
        Binding(
            get: { app.isAuthEnabled },
            set: { app.dispatch(action: .toggleAuth) }
        )
    }

    var useBiometric: Binding<Bool> {
        Binding(
            get: { app.authType == .both || app.authType == .biometric },
            set: { app.dispatch(action: .toggleBiometric) }
        )
    }

    var body: some View {
        Form {
            Section(header: Text("Network")) {
                Picker(
                    "Network",
                    selection: Binding(
                        get: { app.selectedNetwork },
                        set: {
                            networkChanged.toggle()
                            app.dispatch(action: .changeNetwork(network: $0))
                        }
                    )
                ) {
                    ForEach(allNetworks(), id: \.self) {
                        Text($0.toString())
                    }
                }
                .pickerStyle(SegmentedPickerStyle())
            }

            Section(header: Text("Appearance")) {
                Picker(
                    "Theme",
                    selection: Binding(
                        get: { app.colorSchemeSelection },
                        set: {
                            app.dispatch(action: .changeColorScheme($0))
                        }
                    )
                ) {
                    ForEach(themes, id: \.self) {
                        Text($0.capitalizedString)
                    }
                }
                .pickerStyle(SegmentedPickerStyle())
            }

            NodeSelectionView()

            Section("Security") {
                Toggle(isOn: useAuth) {
                    Label("Require Authentication", systemImage: "lock.shield")
                }

                if useAuth {
                    if canUseBiometrics() {
                        Toggle(isOn: $useBiometric) {
                            Label("Enable Face ID", systemImage: "faceid")
                        }
                    }

                    Toggle(isOn: $usePIN) {
                        Label("Enable PIN", systemImage: "key.fill")
                    }
                }
            }

            Section(header: Text("About")) {
                HStack {
                    Text("Version")
                    Spacer()
                    Text("0.0.0")
                        .foregroundColor(.secondary)
                }
            }
        }
        .navigationTitle("Settings")
        .navigationBarBackButtonHidden(networkChanged)
        .toolbar {
            networkChanged
                ? ToolbarItem(placement: .navigationBarLeading) {
                    Button(action: {
                        if networkChanged {
                            showConfirmationAlert = true
                        } else {
                            dismiss()
                        }
                    }) {
                        HStack(spacing: 0) {
                            Image(systemName: "chevron.left")
                                .fontWeight(.semibold)
                                .padding(.horizontal, 0)
                            Text("Back")
                                .offset(x: 5)
                        }
                        .offset(x: -8)
                    }
                } : nil
        }
        .alert(isPresented: $showConfirmationAlert) {
            Alert(
                title: Text("⚠️ Network Changed ⚠️"),
                message: Text("You've changed your network to \(app.selectedNetwork)"),
                primaryButton: .destructive(Text("Yes, Change Network")) {
                    app.resetRoute(to: .listWallets)
                    dismiss()
                },
                secondaryButton: .cancel(Text("Cancel"))
            )
        }
        .preferredColorScheme(app.colorScheme)
        .gesture(
            networkChanged
                ? DragGesture()
                .onChanged { gesture in
                    if gesture.startLocation.x < 25, gesture.translation.width > 100 {
                        withAnimation(.spring()) {
                            showConfirmationAlert = true
                        }
                    }
                }
                .onEnded { gesture in
                    if gesture.startLocation.x < 20, gesture.translation.width > 50 {
                        withAnimation(.spring()) {
                            showConfirmationAlert = true
                        }
                    }
                } : nil
        )
    }
}

#Preview {
    SettingsScreen()
        .environment(AppManager())
}
