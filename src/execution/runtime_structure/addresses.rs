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

use alloc::vec::Vec;
use core::{cmp::Ordering, marker::PhantomData};

/// A trait for all address types.
pub(crate) trait Addr: Copy + core::fmt::Debug + core::fmt::Display + Eq {
    fn new(inner: usize) -> Self;

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
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given address is valid in this vector.
    pub unsafe fn get(&self, addr: A) -> &Inst {
        let addr = addr.into_inner();

        debug_assert!(self.inner.get(addr).is_some());
        // SAFETY: The caller ensures that the given address is valid in this vector. Because this
        // vector cannot shrink the address must point to an existing element.
        unsafe { self.inner.get_unchecked(addr) }
    }

    /// Returns a mutable reference to some instance by its address `addr`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given address is valid in this vector.
    pub unsafe fn get_mut(&mut self, addr: A) -> &mut Inst {
        let addr = addr.into_inner();

        debug_assert!(self.inner.get_mut(addr).is_some());
        // SAFETY: The caller ensures that the given address is valid in this vector. Because this
        // vector cannot shrink, the address must still be valid.
        unsafe { self.inner.get_unchecked_mut(addr) }
    }

    /// Inserts a new instance into the current [`Store`](crate::Store) and returns its address.
    ///
    /// This method should always be used to insert new instances, as it is the only safe way of creating addrs.
    pub fn insert(&mut self, instance: Inst) -> A {
        let new_addr = self.inner.len();
        self.inner.push(instance);
        A::new(new_addr)
    }

    /// Mutably borrows two instances by their addresses and returns those
    /// references. In the case where both given addresses are equal, `None` is
    /// returned instead.
    ///
    /// # Safety
    ///
    /// The caller must ensure that both given addresses are valid in this
    /// vector.
    pub unsafe fn get_two_mut(
        &mut self,
        addr_one: A,
        addr_two: A,
    ) -> Option<(&mut Inst, &mut Inst)> {
        let addr_one = addr_one.into_inner();
        let addr_two = addr_two.into_inner();

        match addr_one.cmp(&addr_two) {
            Ordering::Greater => {
                debug_assert!(self.inner.get(addr_one).is_some());
                // SAFETY: The caller ensures that the given address is valid in this vector.
                // Because this vector cannot shrink, the address must still point to an existing
                // element.
                let (left, right) = unsafe { self.inner.split_at_mut_unchecked(addr_one) };

                debug_assert!(!right.is_empty());
                // SAFETY: `right` starts with the element pointed to by `addr_one`, which was valid
                // in this vector. Therefore, `right` must contain at least one element.
                let one = unsafe { right.get_unchecked_mut(0) };

                debug_assert!(left.get(addr_two).is_some());
                // SAFETY: `left` contains the first `addr_one` elements from this vector. Because
                // `addr_one` is greater than `addr_two`, `addr_two` must be a valid index in
                // `left`.
                let two = unsafe { left.get_unchecked_mut(addr_two) };

                Some((one, two))
            }
            Ordering::Less => {
                debug_assert!(self.inner.get(addr_two).is_some());
                // SAFETY: The caller ensures that the given address is valid in this vector.
                // Because this vector cannot shrink, the address must still point to an existing
                // element.
                let (left, right) = unsafe { self.inner.split_at_mut_unchecked(addr_two) };

                debug_assert!(left.get(addr_one).is_some());
                // SAFETY: `left` contains the first `addr_two` elements from this vector. Because
                // `addr_one` is less than `addr_two`, `addr_one` must be a valid index in `left`.
                let one = unsafe { left.get_unchecked_mut(addr_one) };

                debug_assert!(!right.is_empty());
                // SAFETY: `right` starts with the element points to by `addr_two`, which was valid
                // in this vector. Therefore, `right` must contain at least one element.
                let two = unsafe { right.get_unchecked_mut(0) };

                Some((one, two))
            }
            Ordering::Equal => None,
        }
    }
}

/// An address to a function instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FuncAddr(usize);

impl core::fmt::Display for FuncAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "function address {}", self.0)
    }
}

impl Addr for FuncAddr {
    fn new(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

/// An address to a table instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TableAddr(usize);

impl core::fmt::Display for TableAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "table address {}", self.0)
    }
}

impl Addr for TableAddr {
    fn new(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
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
    fn new(inner: usize) -> Self {
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
    fn new(inner: usize) -> Self {
        Self(inner)
    }

    /// Returns the inner integer represented by this [`GlobalAddr`].
    fn into_inner(self) -> usize {
        self.0
    }
}

/// An address to an element instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ElemAddr(usize);

impl core::fmt::Display for ElemAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "element segment address {}", self.0)
    }
}

impl Addr for ElemAddr {
    fn new(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

/// An address to a data instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DataAddr(usize);

impl core::fmt::Display for DataAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "data segment address {}", self.0)
    }
}

impl Addr for DataAddr {
    fn new(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}

/// An address to a module instance that lives in a specific [`Store`](crate::Store).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ModuleAddr(usize);

impl core::fmt::Display for ModuleAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "module address {}", self.0)
    }
}

impl Addr for ModuleAddr {
    fn new(inner: usize) -> Self {
        Self(inner)
    }

    fn into_inner(self) -> usize {
        self.0
    }
}
