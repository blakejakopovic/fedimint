//! Fedimint Core API (common) module interface
//!
//! Fedimint supports externally implemented modules.
//!
//! This (Rust) module defines common interoperability types
//! and functionality that is used on both client and sever side.
use std::io;
use std::{any::Any, collections::BTreeMap};

pub use bitcoin::KeyPair;
use fedimint_api::{
    dyn_newtype_define, dyn_newtype_impl_dyn_clone_passhthrough,
    encoding::{Decodable, DecodeError, DynEncodable, Encodable},
    Amount,
};

pub mod encode;

pub mod client;
pub mod server;

/// A module key identifing a module
///
/// Used as an unique ID, and also as prefix in serialization
/// of module-specific data.
pub type ModuleKey = u16;

/// Temporary constant for the modules we already have
///
/// To be removed after modularization is complete.
pub const MODULE_KEY_WALLET: u16 = 0;
pub const MODULE_KEY_MINT: u16 = 1;
pub const MODULE_KEY_LN: u16 = 2;
// not really a module
pub const MODULE_KEY_GLOBAL: u16 = 1024;

/// Implement `Encodable` and `Decodable` for a "module dyn newtype"
///
/// "Module dyn newtype" is just a "dyn newtype" used by general purpose
/// Fedimint code to abstract away details of mint modules.
#[macro_export]
macro_rules! module_dyn_newtype_impl_encode_decode {
    (
        $name:ident, $decode_fn:ident
    ) => {
        impl Encodable for $name {
            fn consensus_encode<W: std::io::Write>(
                &self,
                writer: &mut W,
            ) -> Result<usize, std::io::Error> {
                self.0.module_key().consensus_encode(writer)?;
                self.0.consensus_encode_dyn(writer)
            }
        }

        impl Decodable for $name {
            fn consensus_decode<M, R: std::io::Read>(
                r: &mut R,
                modules: &BTreeMap<ModuleKey, M>,
            ) -> Result<Self, DecodeError>
            where
                M: $crate::core::ModuleDecode,
            {
                $crate::core::encode::module_decode_key_prefixed_decodable(r, modules, |r, m| {
                    m.$decode_fn(r)
                })
            }
        }
    };
}

/// Define a "plugin" trait
///
/// "Plugin trait" is a trait that a developer of a mint module
/// needs to implement when implementing mint module. It uses associated
/// types with trait bonds to guide the developer.
///
/// Blanket implementations are used to convert the "plugin trait",
/// incompatible with `dyn Trait` into "module types" and corresponding
/// "module dyn newtypes", erasing the exact type and used in a common
/// Fedimint code.
#[macro_export]
macro_rules! module_plugin_trait_define {
    (   $(#[$outer:meta])*
        $newtype_ty:ident, $plugin_ty:ident, $module_ty:ident, { $($extra_methods:tt)*  } { $($extra_impls:tt)* }
    ) => {
        pub trait $plugin_ty:
            DynEncodable + Decodable + Encodable + Clone + Send + Sync + 'static
        {
            fn module_key(&self) -> ModuleKey;

            $($extra_methods)*
        }

        impl<T> $module_ty for T
        where
            T: $plugin_ty + DynEncodable + 'static,
        {
            fn as_any(&self) -> &(dyn Any + 'static) {
                self
            }

            fn module_key(&self) -> ModuleKey {
                <Self as $plugin_ty>::module_key(self)
            }

            fn clone(&self) -> $newtype_ty {
                <Self as Clone>::clone(self).into()
            }

            $($extra_impls)*
        }
    };
}

/// Module Decoder trait
///
/// Static-polymorphism version of [`ModuleDecode`]
///
/// All methods are static, as the decoding code is supposed to be instance-independent,
/// at least until we start to support modules with overriden [`ModuleKey`]s
pub trait PluginDecode {
    /// Decode `SpendableOutput` compatible with this module, after the module key prefix was already decoded
    fn decode_spendable_output(r: &mut dyn io::Read) -> Result<SpendableOutput, DecodeError>;

    /// Decode `Input` compatible with this module, after the module key prefix was already decoded
    fn decode_input(r: &mut dyn io::Read) -> Result<Input, DecodeError>;

    /// Decode `Output` compatible with this module, after the module key prefix was already decoded
    fn decode_output(r: &mut dyn io::Read) -> Result<Output, DecodeError>;

    /// Decode `PendingOutput` compatible with this module, after the module key prefix was already decoded
    fn decode_pending_output(r: &mut dyn io::Read) -> Result<PendingOutput, DecodeError>;

    /// Decode `OutputOutcome` compatible with this module, after the module key prefix was already decoded
    fn decode_output_outcome(r: &mut dyn io::Read) -> Result<OutputOutcome, DecodeError>;

    /// Decode `ConsensusItem` compatible with this module, after the module key prefix was already decoded
    fn decode_consensus_item(r: &mut dyn io::Read) -> Result<ConsensusItem, DecodeError>;
}

pub trait ModuleDecode {
    /// Decode `SpendableOutput` compatible with this module, after the module key prefix was already decoded
    fn decode_spendable_output(&self, r: &mut dyn io::Read)
        -> Result<SpendableOutput, DecodeError>;

    /// Decode `Input` compatible with this module, after the module key prefix was already decoded
    fn decode_input(&self, r: &mut dyn io::Read) -> Result<Input, DecodeError>;

    /// Decode `Output` compatible with this module, after the module key prefix was already decoded
    fn decode_output(&self, r: &mut dyn io::Read) -> Result<Output, DecodeError>;

    /// Decode `PendingOutput` compatible with this module, after the module key prefix was already decoded
    fn decode_pending_output(&self, r: &mut dyn io::Read) -> Result<PendingOutput, DecodeError>;

    /// Decode `OutputOutcome` compatible with this module, after the module key prefix was already decoded
    fn decode_output_outcome(&self, r: &mut dyn io::Read) -> Result<OutputOutcome, DecodeError>;

    /// Decode `ConsensusItem` compatible with this module, after the module key prefix was already decoded
    fn decode_consensus_item(&self, r: &mut dyn io::Read) -> Result<ConsensusItem, DecodeError>;
}

impl ModuleDecode for () {
    fn decode_spendable_output(
        &self,
        _r: &mut dyn io::Read,
    ) -> Result<SpendableOutput, DecodeError> {
        panic!("() is just a placeholder for when modules are not needed and should never be actually called");
    }
    fn decode_input(&self, _r: &mut dyn io::Read) -> Result<Input, DecodeError> {
        panic!("() is just a placeholder for when modules are not needed and should never be actually called");
    }

    fn decode_output(&self, _r: &mut dyn io::Read) -> Result<Output, DecodeError> {
        panic!("() is just a placeholder for when modules are not needed and should never be actually called");
    }

    fn decode_pending_output(&self, _r: &mut dyn io::Read) -> Result<PendingOutput, DecodeError> {
        panic!("() is just a placeholder for when modules are not needed and should never be actually called");
    }

    fn decode_output_outcome(&self, _r: &mut dyn io::Read) -> Result<OutputOutcome, DecodeError> {
        panic!("() is just a placeholder for when modules are not needed and should never be actually called");
    }

    fn decode_consensus_item(&self, _r: &mut dyn io::Read) -> Result<ConsensusItem, DecodeError> {
        panic!("() is just a placeholder for when modules are not needed and should never be actually called");
    }
}

/// Something that can be an [`Input`] in a [`Transaction`]
///
/// General purpose code should use [`Input`] instead
pub trait ModuleInput: DynEncodable {
    fn as_any(&self) -> &(dyn Any + 'static);
    fn module_key(&self) -> ModuleKey;
    fn amount(&self) -> Amount;
    fn clone(&self) -> Input;
}

module_plugin_trait_define! {
    Input, PluginInput, ModuleInput,
    {
        fn amount(&self) -> Amount;
    }
    {
        fn amount(&self) -> Amount {
            <Self as PluginInput>::amount(self)
        }
    }
}

dyn_newtype_define! {
    /// An owned, immutable input to a [`Transaction`]
    pub Input(Box<ModuleInput>)
}
module_dyn_newtype_impl_encode_decode! {
    Input, decode_input
}
dyn_newtype_impl_dyn_clone_passhthrough!(Input);

/// Something that can be an [`Output`] in a [`Transaction`]
///
/// General purpose code should use [`Output`] instead
pub trait ModuleOutput: DynEncodable {
    fn as_any(&self) -> &(dyn Any + 'static);
    fn module_key(&self) -> ModuleKey;
    fn amount(&self) -> Amount;

    fn clone(&self) -> Output;
}

dyn_newtype_define! {
    /// An owned, immutable output of a [`Transaction`]
    pub Output(Box<ModuleOutput>)
}
module_plugin_trait_define! {
    Output, PluginOutput, ModuleOutput,
    {
        fn amount(&self) -> Amount;
    }
    {
        fn amount(&self) -> Amount {
            <Self as PluginOutput>::amount(self)
        }
    }
}
module_dyn_newtype_impl_encode_decode! {
    Output, decode_output
}
dyn_newtype_impl_dyn_clone_passhthrough!(Output);

/// A spendable output - tracked and persisted by the client
///
/// Created by generating transaction [`Output`], spendable
/// by converting to [`Input`].
pub trait ModuleSpendableOutput: DynEncodable {
    fn as_any(&self) -> &(dyn Any + '_);
    /// Module key
    fn module_key(&self) -> ModuleKey;
    fn amount(&self) -> Amount;
    fn clone(&self) -> SpendableOutput;

    // TODO: move to be module function
    /// Prepare [`Input`] spending thish output in a transaction, and a key used to sign the [`Transaction`]
    // fn to_input(&self) -> (Input, KeyPair);

    fn key(&self) -> String;
}

dyn_newtype_define! {
    /// An owned, immutable output of a [`Transaction`] after it was finalized (so it's spendable)
    pub SpendableOutput(Box<ModuleSpendableOutput>)
}
module_plugin_trait_define! {
    SpendableOutput, PluginSpendableOutput, ModuleSpendableOutput,
    {
        fn amount(&self) -> Amount;
        fn key(&self) -> String;
    }
    {
        fn amount(&self) -> Amount {
            <Self as PluginSpendableOutput>::amount(self)
        }
        fn key(&self) -> String {
            <Self as PluginSpendableOutput>::key(self)
        }
    }
}
module_dyn_newtype_impl_encode_decode! {
    SpendableOutput, decode_spendable_output
}
dyn_newtype_impl_dyn_clone_passhthrough!(SpendableOutput);

pub enum FinalizationError {
    SomethingWentWrong,
}

/// A pending output - tracked and persisted by the client
///
/// Created by generating transaction [`Output`], spendable
/// by converting to [`Input`].
pub trait ModulePendingOutput: DynEncodable {
    fn as_any(&self) -> &(dyn Any + 'static);
    /// Module key
    fn module_key(&self) -> ModuleKey;
    fn amount(&self) -> Amount;
    fn clone(&self) -> PendingOutput;

    // fn key(&self) -> String;
}

dyn_newtype_define! {
    /// An owned, immutable output of a [`Transaction`] before it was finalized
    pub PendingOutput(Box<ModulePendingOutput>)
}
module_plugin_trait_define! {
    PendingOutput, PluginPendingOutput, ModulePendingOutput,
    {
        fn amount(&self) -> Amount;
    }
    {
        fn amount(&self) -> Amount {
            <Self as PluginPendingOutput>::amount(self)
        }
    }
}
module_dyn_newtype_impl_encode_decode! {
    PendingOutput, decode_pending_output
}
dyn_newtype_impl_dyn_clone_passhthrough!(PendingOutput);

pub trait ModuleOutputOutcome: DynEncodable {
    fn as_any(&self) -> &(dyn Any + '_);
    /// Module key
    fn module_key(&self) -> ModuleKey;
    fn clone(&self) -> OutputOutcome;
}

dyn_newtype_define! {
    /// An owned, immutable output of a [`Transaction`] before it was finalized
    pub OutputOutcome(Box<ModuleOutputOutcome>)
}
module_plugin_trait_define! {
    OutputOutcome, PluginOutputOutcome, ModuleOutputOutcome,
    { }
    { }
}
module_dyn_newtype_impl_encode_decode! {
    OutputOutcome, decode_output_outcome
}
dyn_newtype_impl_dyn_clone_passhthrough!(OutputOutcome);

pub trait ModuleConsensusItem: DynEncodable {
    fn as_any(&self) -> &(dyn Any + 'static);
    /// Module key
    fn module_key(&self) -> ModuleKey;
    fn clone(&self) -> ConsensusItem;
}

dyn_newtype_define! {
    /// An owned, immutable output of a [`Transaction`] before it was finalized
    pub ConsensusItem(Box<ModuleConsensusItem>)
}
module_plugin_trait_define! {
    ConsensusItem, PluginConsensusItem, ModuleConsensusItem,
    { }
    { }
}
module_dyn_newtype_impl_encode_decode! {
    ConsensusItem, decode_consensus_item
}
dyn_newtype_impl_dyn_clone_passhthrough!(ConsensusItem);

#[derive(Encodable, Decodable)]
pub struct Signature;

/// Transaction that was already signed
#[derive(Encodable)]
pub struct Transaction {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    signature: Signature,
}

impl Decodable for Transaction
where
    Input: Decodable,
    Output: Decodable,
{
    fn consensus_decode<M, R: std::io::Read>(
        r: &mut R,
        modules: &BTreeMap<ModuleKey, M>,
    ) -> Result<Self, DecodeError>
    where
        M: ModuleDecode,
    {
        Ok(Self {
            inputs: Decodable::consensus_decode(r, modules)?,
            outputs: Decodable::consensus_decode(r, modules)?,
            signature: Decodable::consensus_decode(r, modules)?,
        })
    }
}
