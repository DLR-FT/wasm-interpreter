//! Type definitions for addr types
//!
//! An addr (short for: address) is a dynamic index only known at runtime into a
//! store. There are addr types for different index spaces, such as memories,
//! globals or functions [`FuncAddr`].
//!
//!
//! # A Note About Accessor Methods on Store Address Spaces
//! At first, we stored a [`Vec`] directly in the [`Store`](crate::Store) for
//! function instances, table instances, etc. However, implementing accessor
//! methods on the [`Store`](crate::Store) causes problems, because either the
//! entire [`Store`](crate::Store) has to be passed as an argument (preventing
//! partial borrows) or a specific [`Vec`] has to be passed as an argument
//! (exposing [`Store`](crate::Store) implementation details through a pretty
//! unergonomic API).
//!
//! Because both of these solutions were not sufficient, a choice was made for
//! newtype wrappers around every address space. This way, partial borrows of
//! the [`Store`](crate::Store) are possible, while providing a nice API, even
//! if it is just used internally.

use super::TableInst;
use core::{cmp::Ordering, marker::PhantomData};

use alloc::vec::Vec;

/// A trait for all address types.
///
/// This is used by [`AddrVec`] to create and read address types.
pub(crate) trait Addr: Copy + core::fmt::Debug + core::fmt::Display + Eq {
    fn new_unchecked(inner: usize) -> Self;

    fn into_inner(self) -> usize;
}

pub(crate) struct AddrVec<A: Addr, Inst> {
    inner: Vec<Inst>,
    _phantom: PhantomData<A>,
}

impl<A: Addr, Inst> Default for AddrVec<A, Inst> {
    fn default() -> Self {
        Self {
            inner: Vec::default(),
            _phantom: PhantomData,
        }
    }
}

impl<A: Addr, Inst> AddrVec<A, Inst> {
    /// Returns an instance by its address `addr`.
    pub fn get(&self, addr: A) -> &Inst {
        self.inner
            .get(addr.into_inner())
            .expect("addrs to always be valid")
    }

    /// Returns a mutable reference to some instance by its address `addr`.
    pub fn get_mut(&mut self, addr: A) -> &mut Inst {
        self.inner
            .get_mut(addr.into_inner())
            .expect("addrs to always be valid")
    }

    /// Inserts a new function instance into the current [`Store`](crate::Store) and returns its addr.
    ///
    /// This method should always be used to insert new instances, as it is the only safe way of creating addrs.
    pub(crate) fn insert(&mut self, instance: Inst) -> A {
        let new_addr = self.inner.len();
        self.inner.push(instance);
        A::new_unchecked(new_addr)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FuncAddr(usize);

impl FuncAddr {
    // This is unfortunately needed as a default value for the base `CallFrame` in every call stack.
    pub(crate) const INVALID: Self = FuncAddr(usize::MAX);

    /// Returns the inner representation of this function address.
    pub fn into_inner(self) -> usize {
        self.0
    }
}

impl core::fmt::Display for FuncAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "function address {}", self.0)
    }
}

impl Addr for FuncAddr {
    fn new_unchecked(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

/// An address to a [`TableInst`] that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TableAddr(usize);

impl core::fmt::Display for TableAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "table address {}", self.0)
    }
}

impl Addr for TableAddr {
    fn new_unchecked(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

impl AddrVec<TableAddr, TableInst> {
    /// Mutably borrows two table instances by their table addresses and
    /// returns those references. In the case where both given table
    /// addresses are equal, `None` is returned instead.
    pub(crate) fn get_two_mut(
        &mut self,
        addr_one: TableAddr,
        addr_two: TableAddr,
    ) -> Option<(&mut TableInst, &mut TableInst)> {
        match addr_one.0.cmp(&addr_two.0) {
            Ordering::Greater => {
                let (left, right) = self.inner.split_at_mut(addr_one.0);
                let one = right.get_mut(0).expect(
                    "this to be exactly the same as addr_one and addresses to always be valid",
                );
                let two = left
                    .get_mut(addr_two.0)
                    .expect("addresses to always be valid");

                Some((one, two))
            }
            Ordering::Less => {
                let (left, right) = self.inner.split_at_mut(addr_two.0);
                let one = left
                    .get_mut(addr_one.0)
                    .expect("addresses to always be valid");
                let two = right.get_mut(0).expect(
                    "this to be exactly the same as addr_two and addresses to always be valid",
                );

                Some((one, two))
            }
            Ordering::Equal => None,
        }
    }
}

/// An address to a memory instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MemAddr(usize);

impl core::fmt::Display for MemAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "memory address {}", self.0)
    }
}

impl Addr for MemAddr {
    fn new_unchecked(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

/// An address to a global instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GlobalAddr(usize);

impl core::fmt::Display for GlobalAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "global address {}", self.0)
    }
}

impl Addr for GlobalAddr {
    fn new_unchecked(inner: usize) -> Self {
        Self(inner)
    }

    /// Returns the inner integer represented by this [`GlobalAddr`].
    fn into_inner(self) -> usize {
        self.0
    }
}
