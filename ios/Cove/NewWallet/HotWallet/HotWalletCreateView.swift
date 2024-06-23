//
//  HotWalletCreateView.swift
//  Cove
//
//  Created by Praveen Perera on 6/18/24.
//

import SwiftUI

struct HotWalletCreateView: View {
    @State private var model: WalletViewModel

    init(numberOfWords: NumberOfBip39Words) {
        self.model = WalletViewModel(numberOfWords: numberOfWords)
    }

    var body: some View {
        WordsView(model: model, groupedWords: model.rust.bip39WordsGrouped())
    }
}

struct WordsView: View {
    var model: WalletViewModel
    var groupedWords: [[GroupedWord]]
    @State private var tabIndex = 0
    @State private var showConfirmationAlert = false
    @Environment(\.presentationMode) var presentationMode

    var lastIndex: Int {
        return groupedWords.count - 1
    }

    var body: some View {
        SunsetWave {
            VStack {
                Spacer()

                Text("Please write these words down")
                    .font(.title2)
                    .fontWeight(.semibold)
                    .foregroundColor(.white.opacity(0.75))
                    .padding(.top, 50)

                StyledWordCard(tabIndex: $tabIndex) {
                    ForEach(Array(groupedWords.enumerated()), id: \.offset) { index, wordGroup in
                        WordCardView(words: wordGroup).tag(index)
                    }
                }
                .frame(height: 400)
                .padding()

                Spacer()

                if tabIndex == lastIndex {
                    Button("Save Wallet") {}
                        .buttonStyle(GradientButtonStyle())
                        .padding(.top, 50)

                } else {
                    Button("Next") {
                        withAnimation {
                            tabIndex += 1
                        }
                    }
                    .buttonStyle(GlassyButtonStyle())
                    .padding(.top, 50)
                }

                Spacer()
            }
        }
        .navigationBarBackButtonHidden(true)
        .toolbar {
            ToolbarItem(placement: .navigationBarLeading) {
                Button(action: {
                    showConfirmationAlert = true
                }) {
                    HStack {
                        Image(systemName: "chevron.left")
                        Text("Back")
                    }
                }
            }
        }
        .alert(isPresented: $showConfirmationAlert) {
            Alert(
                title: Text("⚠️ Wallet Not Saved ⚠️"),
                message: Text("You will have to write down a new set of words."),
                primaryButton: .destructive(Text("Yes, Go Back")) {
                    presentationMode.wrappedValue.dismiss()
                },
                secondaryButton: .cancel(Text("Cancel"))
            )
        }
    }
}

struct WordCardView: View {
    let words: [GroupedWord]

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ForEach(words, id: \.self) { group in
                HStack {
                    Text("\(group.number).")
                        .foregroundColor(.secondary)
                        .frame(width: 30, alignment: .trailing)
                        .padding(.trailing, 8)
                        .multilineTextAlignment(.center)

                    Text(group.word)
                        .font(.headline)
                }
            }
        }
        .padding()
        .foregroundColor(.white)
    }
}

struct StyledWordCard<Content: View>: View {
    @Binding var tabIndex: Int
    @ViewBuilder var content: Content

    var body: some View {
        GlassCard {
            TabView(selection: $tabIndex) {
                content
            }
            .tabViewStyle(PageTabViewStyle(indexDisplayMode: .automatic))
        }
        .padding()
    }
}

#Preview("12 Words") {
    HotWalletCreateView(numberOfWords: .twelve)
}

#Preview("24 Words") {
    HotWalletCreateView(numberOfWords: .twentyFour)
}
