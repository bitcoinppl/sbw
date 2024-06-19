use std::sync::Arc;

use crossbeam::channel::{Receiver, Sender};
use parking_lot::RwLock;

use crate::wallet::NumberOfBip39Words;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, uniffi::Enum)]
pub enum WalletViewModelReconcileMessage {
    Words(NumberOfBip39Words),
}

#[uniffi::export(callback_interface)]
pub trait WalletViewModelReconciler: Send + Sync + std::fmt::Debug + 'static {
    /// Tells the frontend to reconcile the view model changes
    fn reconcile(&self, message: WalletViewModelReconcileMessage);
}

#[derive(Clone, Debug, uniffi::Object)]
pub struct RustWalletViewModel {
    pub state: Arc<RwLock<WalletViewModelState>>,
    pub reconciler: Sender<WalletViewModelReconcileMessage>,
    pub reconcile_receiver: Arc<Receiver<WalletViewModelReconcileMessage>>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct WalletViewModelState {
    pub words: NumberOfBip39Words,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, uniffi::Enum)]
pub enum WalletViewModelAction {
    UpdateWords(NumberOfBip39Words),
}

#[uniffi::export]
impl RustWalletViewModel {
    #[uniffi::constructor]
    pub fn new(words: NumberOfBip39Words) -> Self {
        let (sender, receiver) = crossbeam::channel::bounded(1000);

        Self {
            state: Arc::new(RwLock::new(WalletViewModelState::new(words))),
            reconciler: sender,
            reconcile_receiver: Arc::new(receiver),
        }
    }

    #[uniffi::method]
    pub fn get_state(&self) -> WalletViewModelState {
        self.state.read().clone()
    }

    #[uniffi::method]
    pub fn listen_for_updates(&self, reconciler: Box<dyn WalletViewModelReconciler>) {
        let reconcile_receiver = self.reconcile_receiver.clone();

        std::thread::spawn(move || {
            while let Ok(field) = reconcile_receiver.recv() {
                // call the reconcile method on the frontend
                reconciler.reconcile(field);
            }
        });
    }

    /// Action from the frontend to change the state of the view model
    #[uniffi::method]
    pub fn dispatch(&self, action: WalletViewModelAction) {
        let state = self.state.clone();

        match action {
            WalletViewModelAction::UpdateWords(words) => {
                state.write().words = words;

                self.reconciler
                    .send(WalletViewModelReconcileMessage::Words(words))
                    .expect("failed to send update");
            }
        }
    }
}

impl WalletViewModelState {
    pub fn new(words: NumberOfBip39Words) -> Self {
        Self { words }
    }
}
