use std::convert::TryFrom;
pub mod encoding;
pub mod server;
pub use encoding::OsuEncode;
pub use uncho_common::packets::Id;

#[macro_use]
macro_rules! count_items {
    () => { 0 };
    ($a:ident) => { 1 };
    ($a:ident, $($b:ident),+) => { 1 + count_items!($($b),+) }
}

macro_rules! impl_osu_encode {
    ($name:ident : $t:ident) => {
        impl $crate::packets::OsuEncode for $name {
            fn encoded_size(&self) -> usize { std::mem::size_of_val(self) }

            fn encode(&self, buf: &mut Vec<u8>) { (*self as $t).encode(buf); }

            fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
                //use $crate::packets::OsuEncode;
                let (&val, off) = <$t>::decode(buf)?;
                use std::convert::TryFrom;
                Self::try_from(val)?;
                Ok((unsafe { &*(buf.as_ptr() as *const Self) }, off))
            }
        }
    };
}

impl_osu_encode!(Id: u16);

#[macro_use]
macro_rules! enum_try_from {
    (
        #[repr($t:ident)]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($field:ident = $val:expr),* $(,)?
        }
    ) => {
       #[repr($t)]
       $(#[$meta])*
       $vis enum $name {
           $($field = $val),*
       }

       impl std::convert::TryFrom<$t> for $name {
           type Error = ();
           fn try_from(value: $t) -> Result<$name, ()> {
               match value {
                   $(
                       $val => Ok($name::$field),
                   )*
                   _ => Err(())
               }
           }
       }

        impl $crate::packets::OsuEncode for $name {
            fn encoded_size(&self) -> usize { std::mem::size_of::<$t>() }

            fn encode(&self, buf: &mut Vec<u8>) {
                (*self as $t).encode(buf);
            }

            fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
                //use $crate::packets::OsuEncode;
                let (&val, off) = <$t>::decode(buf)?;
                use std::convert::TryFrom;
                Self::try_from(val)?;
                Ok((
                    unsafe { &*(buf.as_ptr() as *const Self) },
                    off
                ))
            }
        }
    };

    (
        #[repr($t:ident)]
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($field:ident),* $(,)?
        }
    ) => {
        #[repr($t)]
        $(#[$meta])*
        $vis enum $name {
            $($field),*
        }

        impl std::convert::TryFrom<$t> for $name {
            type Error = ();
            fn try_from(value: $t) -> Result<$name, ()> {
                const MAX: $t = count_items!($($field),*) - 1;
                if value > MAX { Err(()) }
                else {
                    Ok(unsafe {
                        std::mem::transmute::<$t, $name>(value)
                    })
                }
            }
        }

        impl $crate::packets::OsuEncode for $name {
            fn encoded_size(&self) -> usize { std::mem::size_of::<$t>() }

            fn encode(&self, buf: &mut Vec<u8>) {
                (*self as $t).encode(buf);
            }

            fn decode(buf: &[u8]) -> Result<(&Self, usize), ()> {
                //use $crate::packets::OsuEncode;
                let (&val, off) = <$t>::decode(buf)?;
                use std::convert::TryFrom;
                Self::try_from(val)?;
                Ok((
                    unsafe { &*(buf.as_ptr() as *const Self) },
                    off
                ))
            }
        }
    }
}

pub fn parse_packet(buf: &[u8]) -> Result<(Id, usize), ()> {
    let (&id, _) = u16::decode(buf)?;
    let buf = &buf[3..];
    let (&len, _) = u32::decode(buf)?;
    if (&buf[4..]).len() < len as usize {
        println!("{:?} {} {}", id, &buf[4..].len(), len);
        return Err(());
    }
    Ok((Id::try_from(id).unwrap_or(Id::Unknown), len as usize))
}
