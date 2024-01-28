// Copyright 2022 Jeff Kim <hiking90@gmail.com>
// SPDX-License-Identifier: Apache-2.0

/*
 * Copyright (C) 2020 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/// Declare a binder interface.
///
/// This is mainly used internally by the AIDL compiler.
#[macro_export]
macro_rules! declare_binder_interface {
    {
        $interface:path[$descriptor:expr] {
            native: $native:ident($on_transact:path),
            proxy: $proxy:ident,
        }
    } => {
        $crate::declare_binder_interface! {
            $interface[$descriptor] {
                native: $native($on_transact),
                proxy: $proxy {},
                stability: rsbinder::Stability::default(),
            }
        }
    };

    {
        $interface:path[$descriptor:expr] {
            native: $native:ident($on_transact:path),
            proxy: $proxy:ident,
            stability: $stability:expr,
        }
    } => {
        $crate::declare_binder_interface! {
            $interface[$descriptor] {
                native: $native($on_transact),
                proxy: $proxy {},
                stability: $stability,
            }
        }
    };

    {
        $interface:path[$descriptor:expr] {
            native: $native:ident($on_transact:path),
            proxy: $proxy:ident {
                $($fname:ident: $fty:ty = $finit:expr),*
            },
        }
    } => {
        $crate::declare_binder_interface! {
            $interface[$descriptor] {
                native: $native($on_transact),
                proxy: $proxy {
                    $($fname: $fty = $finit),*
                },
                stability: $crate::Stability::default(),
            }
        }
    };

    {
        $interface:path[$descriptor:expr] {
            native: $native:ident($on_transact:path),
            proxy: $proxy:ident {
                $($fname:ident: $fty:ty = $finit:expr),*
            },
            stability: $stability:expr,
        }
    } => {
        $crate::declare_binder_interface! {
            $interface[$descriptor] {
                @doc[concat!("A binder [`Remotable`]($crate::binder_impl::Remotable) that holds an [`", stringify!($interface), "`] object.")]
                native: $native($on_transact),
                @doc[concat!("A binder [`Proxy`]($crate::binder_impl::Proxy) that holds an [`", stringify!($interface), "`] remote interface.")]
                proxy: $proxy {
                    $($fname: $fty = $finit),*
                },
                stability: $stability,
            }
        }
    };

    {
        $interface:path[$descriptor:expr] {
            @doc[$native_doc:expr]
            native: $native:ident($on_transact:path),

            @doc[$proxy_doc:expr]
            proxy: $proxy:ident {
                $($fname:ident: $fty:ty = $finit:expr),*
            },

            stability: $stability:expr,
        }
    } => {
        #[doc = $proxy_doc]
        pub struct $proxy {
            binder: $crate::SIBinder,
            $($fname: $fty,)*
        }

        impl $crate::Interface for $proxy {
            fn as_binder(&self) -> $crate::SIBinder {
                self.binder.clone()
            }
            fn dump(&self, writer: &mut dyn $crate::WriteExt, args: &[String]) -> $crate::Result<()> {
                let proxy = self.binder.as_any().downcast_ref::<$crate::ProxyHandle>().ok_or($crate::StatusCode::BadType)?;
                proxy.dump(writer, args)
            }
        }

        impl $crate::Proxy for $proxy
        where
            $proxy: $interface,
        {
            fn descriptor() -> &'static str {
                $descriptor
            }

            fn from_binder(binder: $crate::SIBinder) -> std::option::Option<Self> {
                if binder.descriptor() != $descriptor {
                    return None
                }
                if let Some(_) = binder.as_proxy() {
                    Some(Self { binder, $($fname: $finit),* })
                } else {
                    None
                }
            }
        }

        pub struct $native(Box<dyn $interface + Sync + Send + 'static>);

        impl $native {
            /// Create a new binder service.
            pub fn new_binder<T: $interface + Sync + Send + 'static>(inner: T) -> $crate::Strong<dyn $interface> {
                let binder = $crate::native::Binder::new_with_stability($native(Box::new(inner)), $stability);
                $crate::Strong::new(Box::new(binder))
            }
        }

        impl $crate::Remotable for $native {
            fn descriptor() -> &'static str where Self: Sized {
                $descriptor
            }

            fn on_transact(&self, code: $crate::TransactionCode, reader: &mut $crate::Parcel, reply: &mut $crate::Parcel) -> $crate::Result<()> {
                $on_transact(&*self.0, code, reader, reply, Self::descriptor())
            }

            fn on_dump(&self, _writer: &mut dyn $crate::WriteExt, _args: &[String]) -> $crate::Result<()> {
                self.0.dump(_writer, _args)
            }
        }

        impl $crate::FromIBinder for dyn $interface {
            fn try_from(binder: $crate::SIBinder) -> $crate::Result<$crate::Strong<dyn $interface>> {
                match <$proxy as $crate::Proxy>::from_binder(binder.clone()) {
                    Some(proxy) => Ok($crate::Strong::new(Box::new(proxy))),
                    None => {
                        match $crate::native::Binder::<$native>::try_from(binder) {
                            Ok(native) => Ok($crate::Strong::new(Box::new(native.clone()))),
                            Err(err) => Err(err),
                        }
                    }
                }
            }
        }

        impl $crate::parcelable::Serialize for dyn $interface
        where
            dyn $interface: $crate::Interface
        {
            fn serialize(&self, parcel: &mut $crate::Parcel) -> $crate::Result<()> {
                let binder = $crate::Interface::as_binder(self);
                parcel.write(&binder)?;
                Ok(())
            }
        }

        impl $crate::parcelable::SerializeOption for dyn $interface {
            fn serialize_option(this: Option<&Self>, parcel: &mut $crate::Parcel) -> $crate::Result<()> {
                parcel.write(&this.map($crate::Interface::as_binder))
            }
        }

        impl std::fmt::Debug for dyn $interface + '_ {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.pad(stringify!($interface))
            }
        }
    }
}


/// Implement `Serialize` trait and friends for a parcelable
///
/// This is an internal macro used by the AIDL compiler to implement
/// `Serialize`, `SerializeArray` and `SerializeOption` for
/// structured parcelables. The target type must implement the
/// `Parcelable` trait.
/// ```
#[macro_export]
macro_rules! impl_serialize_for_parcelable {
    ($parcelable:ident) => {
        impl $crate::Serialize for $parcelable {
            fn serialize(
                &self,
                parcel: &mut $crate::Parcel,
            ) -> $crate::Result<()> {
                <Self as $crate::SerializeOption>::serialize_option(Some(self), parcel)
            }
        }

        impl $crate::SerializeArray for $parcelable {}

        impl $crate::SerializeOption for $parcelable {
            fn serialize_option(
                this: Option<&Self>,
                parcel: &mut $crate::Parcel,
            ) -> $crate::Result<()> {
                if let Some(this) = this {
                    use $crate::Parcelable;
                    parcel.write(&$crate::NON_NULL_PARCELABLE_FLAG)?;
                    this.write_to_parcel(parcel)
                } else {
                    parcel.write(&$crate::NULL_PARCELABLE_FLAG)
                }
            }
        }
    };
}


/// Implement `Deserialize` trait and friends for a parcelable
///
/// This is an internal macro used by the AIDL compiler to implement
/// `Deserialize`, `DeserializeArray` and `DeserializeOption` for
/// structured parcelables. The target type must implement the
/// `Parcelable` trait.
#[macro_export]
macro_rules! impl_deserialize_for_parcelable {
    ($parcelable:ident) => {
        impl $crate::Deserialize for $parcelable {
            fn deserialize(
                parcel: &mut $crate::Parcel,
            ) -> $crate::Result<Self> {
                $crate::DeserializeOption::deserialize_option(parcel)
                    .transpose()
                    .unwrap_or(Err($crate::StatusCode::UnexpectedNull.into()))
            }
            fn deserialize_from(
                &mut self,
                parcel: &mut $crate::Parcel,
            ) -> $crate::Result<()> {
                let status: i32 = parcel.read()?;
                if status == $crate::NULL_PARCELABLE_FLAG {
                    Err($crate::StatusCode::UnexpectedNull.into())
                } else {
                    use $crate::Parcelable;
                    self.read_from_parcel(parcel)
                }
            }
        }

        impl $crate::DeserializeArray for $parcelable {}

        impl $crate::DeserializeOption for $parcelable {
            fn deserialize_option(
                parcel: &mut $crate::Parcel,
            ) -> $crate::Result<Option<Self>> {
                let mut result = None;
                Self::deserialize_option_from(&mut result, parcel)?;
                Ok(result)
            }
            fn deserialize_option_from(
                this: &mut Option<Self>,
                parcel: &mut $crate::Parcel,
            ) -> $crate::Result<()> {
                let status: i32 = parcel.read()?;
                if status == $crate::NULL_PARCELABLE_FLAG {
                    *this = None;
                    Ok(())
                } else {
                    use $crate::Parcelable;
                    this.get_or_insert_with(Self::default)
                        .read_from_parcel(parcel)
                }
            }
        }
    };
}


/// Declare an AIDL enumeration.
///
/// This is mainly used internally by the AIDL compiler.
#[macro_export]
macro_rules! declare_binder_enum {
    {
        $( #[$attr:meta] )*
        $enum:ident : [$backing:ty; $size:expr] {
            $( $( #[$value_attr:meta] )* $name:ident = $value:expr, )*
        }
    } => {
        $( #[$attr] )*
        #[derive(Debug, Default, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
        #[allow(missing_docs)]
        pub struct $enum(pub $backing);
        impl $enum {
            $( $( #[$value_attr] )* #[allow(missing_docs)] pub const $name: Self = Self($value); )*

            #[inline(always)]
            #[allow(missing_docs)]
            pub const fn enum_values() -> [Self; $size] {
                [$(Self::$name),*]
            }
        }

        impl $crate::Serialize for $enum {
            fn serialize(&self, parcel: &mut $crate::Parcel) -> $crate::Result<()> {
                parcel.write(&self.0)
            }
        }

        impl $crate::SerializeArray for $enum {
            fn serialize_array(slice: &[Self], parcel: &mut $crate::Parcel) -> $crate::Result<()> {
                let v: Vec<$backing> = slice.iter().map(|x| x.0).collect();
                <$backing as $crate::SerializeArray>::serialize_array(&v[..], parcel)
            }
        }

        impl $crate::Deserialize for $enum {
            fn deserialize(parcel: &mut $crate::Parcel) -> $crate::Result<Self> {
                let res = parcel.read().map(Self);
                res
            }
        }

        impl $crate::DeserializeArray for $enum {
            fn deserialize_array(parcel: &mut $crate::Parcel) -> $crate::Result<Option<Vec<Self>>> {
                let v: Option<Vec<$backing>> =
                    <$backing as $crate::DeserializeArray>::deserialize_array(parcel)?;
                Ok(v.map(|v| v.into_iter().map(Self).collect()))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{Interface, TransactionCode, Result, Binder, Parcel};

    pub trait IEcho: Interface {
        fn echo(&self, echo: &str) -> Result<String>;
    }

    declare_binder_interface! {
        IEcho["my.echo"] {
            native: BnEcho(on_transact),
            proxy: BpEcho{},
        }
    }

    impl IEcho for Binder<BnEcho> {
        fn echo(&self, echo: &str) -> Result<String> {
            self.0.echo(echo)
        }
    }

    impl IEcho for BpEcho {
        fn echo(&self, _echo: &str) -> Result<String> {
            unimplemented!("BpEcho::echo")
        }
    }

    fn on_transact(
        _service: &dyn IEcho,
        _code: TransactionCode,
        _data: &mut Parcel,
        _reply: &mut Parcel,
        _descriptor: &str,
    ) -> Result<()> {
        // ...
        Ok(())
    }

    struct EchoService {}

    impl Interface for EchoService {}

    impl IEcho for EchoService {
        fn echo(&self, echo: &str) -> Result<String> {
            Ok(echo.to_owned())
        }
    }

    #[test]
    fn test_declare_binder_interface() {
        let _ = BnEcho::new_binder(EchoService {});
    }

}
