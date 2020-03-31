#![no_std]

use new_array::NewArray;

#[derive(NewArray, Clone)]
#[new_array(derive(
    Default, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, AsMut, From, Into, Borrow, BorrowMut,
    Debug, Drop
))]
#[new_array(derive_with_deps(Display))]
pub struct ByteN(pub [u8; 33]);

#[derive(Debug)]
pub struct Byte2([u8; 2]);
