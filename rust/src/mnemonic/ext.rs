use bdk_wallet::{
    bitcoin::{bip32::Xpub, key::Secp256k1, Network},
    keys::{DerivableKey as _, ExtendedKey},
};

use bip39::Mnemonic;

use super::MnemonicExt;
use crate::{keys::Descriptors, wallet::WalletAddressType};

impl MnemonicExt for Mnemonic {
    fn into_descriptors(
        self,
        passphrase: Option<String>,
        network: impl Into<crate::network::Network>,
        address_type: WalletAddressType,
    ) -> Descriptors {
        use crate::keys::{Descriptor, DescriptorSecretKey};

        let network = network.into();
        let descriptor_secret_key = DescriptorSecretKey::new(network, self, passphrase);

        let new_descriptor = match address_type {
            WalletAddressType::NativeSegwit => Descriptor::new_bip84,
            WalletAddressType::WrappedSegwit => Descriptor::new_bip49,
            WalletAddressType::Legacy => Descriptor::new_bip44,
        };

        let descriptor = new_descriptor(
            &descriptor_secret_key,
            bdk_wallet::KeychainKind::External,
            network,
        );

        let change_descriptor = Descriptor::new_bip84(
            &descriptor_secret_key,
            bdk_wallet::KeychainKind::Internal,
            network,
        );

        Descriptors {
            external: descriptor,
            internal: change_descriptor,
        }
    }

    fn xpub(&self, network: Network) -> Xpub {
        let seed = self.to_seed("");
        let xkey: ExtendedKey = seed
            .into_extended_key()
            .expect("never fail proper mnemonic");

        xkey.into_xpub(network, &Secp256k1::new())
    }
}
