use core::convert::{TryFrom, TryInto};
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter::FusedIterator;
use core::str;

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;

use crate::erts::exception::runtime;

use super::*;

pub struct PartialByteBitIter {
    original: Term,
    current_byte_offset: usize,
    current_bit_offset: u8,
    max_byte_offset: usize,
    max_bit_offset: u8,
}

impl Iterator for PartialByteBitIter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if (self.current_byte_offset == self.max_byte_offset)
            & (self.current_bit_offset == self.max_bit_offset)
        {
            None
        } else {
            let byte = match self.original.to_typed_term().unwrap() {
                TypedTerm::Boxed(boxed) => match boxed.to_typed_term().unwrap() {
                    TypedTerm::ProcBin(proc_bin) => proc_bin.byte(self.current_byte_offset),
                    TypedTerm::HeapBinary(heap_binary) => {
                        heap_binary.byte(self.current_byte_offset)
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };
            let bit = (byte >> (7 - self.current_bit_offset)) & 0b1;

            if self.current_bit_offset == 7 {
                self.current_bit_offset = 0;
                self.current_byte_offset += 1;
            } else {
                self.current_bit_offset += 1;
            }

            Some(bit)
        }
    }
}

pub struct ByteIter {
    original: Term,
    base_byte_offset: usize,
    bit_offset: u8,
    current_byte_offset: usize,
    max_byte_offset: usize,
}

impl ByteIter {
    fn is_aligned(&self) -> bool {
        self.bit_offset == 0
    }

    fn byte(&self, index: usize) -> u8 {
        let first_index = self.base_byte_offset + index;

        match self.original.to_typed_term().unwrap() {
            TypedTerm::Boxed(boxed) => match boxed.to_typed_term().unwrap() {
                TypedTerm::ProcBin(proc_bin) => {
                    let first_byte = proc_bin.byte(first_index);

                    if self.is_aligned() {
                        first_byte
                    } else {
                        let second_byte = proc_bin.byte(first_index + 1);

                        (first_byte << self.bit_offset) | (second_byte >> (8 - self.bit_offset))
                    }
                }
                TypedTerm::HeapBinary(heap_binary) => {
                    let first_byte = heap_binary.byte(first_index);

                    if 0 < self.bit_offset {
                        let second_byte = heap_binary.byte(first_index + 1);

                        (first_byte << self.bit_offset) | (second_byte >> (8 - self.bit_offset))
                    } else {
                        first_byte
                    }
                }
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}

impl ByteIterator for ByteIter {}

impl ExactSizeIterator for ByteIter {}

impl DoubleEndedIterator for ByteIter {
    fn next_back(&mut self) -> Option<u8> {
        if self.current_byte_offset == self.max_byte_offset {
            None
        } else {
            self.max_byte_offset -= 1;
            let byte = self.byte(self.max_byte_offset);

            Some(byte)
        }
    }
}

impl FusedIterator for ByteIter {}

impl Iterator for ByteIter {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.current_byte_offset == self.max_byte_offset {
            None
        } else {
            let byte = self.byte(self.current_byte_offset);
            self.current_byte_offset += 1;

            Some(byte)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.max_byte_offset - self.current_byte_offset;

        (size, Some(size))
    }
}

trait ByteIterator: ExactSizeIterator + DoubleEndedIterator + Iterator<Item = u8>
where
    Self: Sized,
{
}

pub trait Original {
    fn byte(&self, index: usize) -> u8;
}

/// A slice of a binary
#[derive(Clone, Copy)]
#[repr(C)]
pub struct SubBinary {
    header: Term,
    /// Byte offset into `original` binary
    byte_offset: usize,
    /// Offset in bits after the `byte_offset`
    bit_offset: u8,
    /// Number of full bytes in binary.  Does not include final byte to store `bit_size` bits.
    full_byte_len: usize,
    /// Number of bits in the partial byte after the `full_byte_len` full bytes.
    partial_byte_bit_len: u8,
    /// Indicates the underlying binary is writable
    writable: bool,
    /// Original binary term (`ProcBin` or `HeapBin`)
    original: Term,
}

impl SubBinary {
    /// See erts_bs_get_binary_2 in erl_bits.c:460
    #[inline]
    pub fn from_match(ctx: &mut MatchContext, bit_len: usize) -> Self {
        assert!(ctx.buffer.bit_len - ctx.buffer.bit_offset < bit_len);

        let original = ctx.buffer.original;
        let subbinary_byte_offset = byte_offset(ctx.buffer.bit_offset);
        let subbinary_bit_offset = bit_offset(ctx.buffer.bit_offset) as u8;
        let full_byte_len = byte_offset(bit_len);
        let partial_byte_bit_len = bit_offset(bit_len) as u8;
        let writable = false;
        ctx.buffer.bit_offset += bit_len as usize;

        Self {
            header: Self::header(),
            original,
            byte_offset: subbinary_byte_offset,
            bit_offset: subbinary_bit_offset,
            full_byte_len,
            partial_byte_bit_len,
            writable,
        }
    }

    #[inline]
    pub fn from_original(
        original: Term,
        byte_offset: usize,
        bit_offset: u8,
        full_byte_len: usize,
        partial_byte_bit_len: u8,
    ) -> Self {
        Self {
            header: Self::header(),
            original,
            byte_offset,
            bit_offset,
            full_byte_len,
            partial_byte_bit_len,
            writable: false,
        }
    }

    #[inline]
    pub unsafe fn from_raw(ptr: *mut SubBinary) -> Self {
        *ptr
    }

    #[inline]
    pub fn bit_offset(&self) -> u8 {
        self.bit_offset
    }

    pub fn is_aligned(&self) -> bool {
        self.bit_offset == 0
    }

    /// Iterator of the [bit_size] bits.  To get the [byte_size] bytes at the beginning of the
    /// bitstring use [byte_iter] if the subbinary may not have [bit_offset] `0` or [as_bytes] has
    /// [bit_offset] `0`.
    pub fn partial_byte_bit_iter(&self) -> PartialByteBitIter {
        let current_byte_offset = self.byte_offset + self.full_byte_len;
        let current_bit_offset = self.bit_offset;

        let improper_bit_offset = current_bit_offset + self.partial_byte_bit_len;
        let max_byte_offset = current_byte_offset + (improper_bit_offset / 8) as usize;
        let max_bit_offset = improper_bit_offset % 8;

        PartialByteBitIter {
            original: self.original,
            current_byte_offset,
            current_bit_offset,
            max_byte_offset,
            max_bit_offset,
        }
    }

    /// Iterator for the [size] bytes.  For the [bit_size] bits in the partial byte at the
    /// end, use [bit_count_iter].
    pub fn byte_iter(&self) -> ByteIter {
        ByteIter {
            original: self.original,
            base_byte_offset: self.byte_offset,
            bit_offset: self.bit_offset,
            current_byte_offset: 0,
            max_byte_offset: self.full_byte_len,
        }
    }

    #[inline]
    pub fn is_binary(&self) -> bool {
        self.partial_byte_bit_len == 0
    }

    #[inline]
    pub fn byte_offset(&self) -> usize {
        self.byte_offset
    }

    #[inline]
    pub fn original(&self) -> Term {
        self.original
    }

    #[inline]
    pub fn bytes(&self) -> *mut u8 {
        let real_bin_ptr = follow_moved(self.original).boxed_val();
        let real_bin = unsafe { *real_bin_ptr };
        if real_bin.is_procbin() {
            let bin = unsafe { &*(real_bin_ptr as *mut ProcBin) };
            bin.bytes()
        } else {
            assert!(real_bin.is_heapbin());
            let bin = unsafe { &*(real_bin_ptr as *mut HeapBin) };
            bin.bytes()
        }
    }

    /// During garbage collection, we sometimes want to convert sub-binary terms
    /// into full-fledged heap binaries, so that the original full-size binary can be freed.
    ///
    /// If this sub-binary is a candidate for conversion, then it will return `Ok((ptr, size))`,
    /// otherwise it will return `Err(())`. The returned pointer and size is sufficient for
    /// passing to `ptr::copy_nonoverlapping` during creation of the new HeapBin.
    ///
    /// NOTE: You should not use this for any other purpose
    pub(crate) fn to_heapbin_parts(&self) -> Result<(Term, usize, *mut u8, usize), ()> {
        if self.is_binary()
            && self.is_aligned()
            && !self.writable
            && self.full_byte_len <= HeapBin::MAX_SIZE
        {
            Ok(unsafe { self.to_raw_parts() })
        } else {
            Err(())
        }
    }

    #[inline]
    unsafe fn to_raw_parts(&self) -> (Term, usize, *mut u8, usize) {
        let real_bin_ptr = follow_moved(self.original).boxed_val();
        let real_bin = *real_bin_ptr;
        if real_bin.is_procbin() {
            let bin = &*(real_bin_ptr as *mut ProcBin);
            let bytes = bin.bytes().offset(self.byte_offset as isize);
            let flags = bin.binary_type().to_flags();
            (bin.header, flags, bytes, self.full_byte_len)
        } else {
            assert!(real_bin.is_heapbin());
            let bin = &*(real_bin_ptr as *mut HeapBin);
            let bytes = bin.bytes().offset(self.byte_offset as isize);
            let flags = bin.binary_type().to_flags();
            (bin.header, flags, bytes, self.full_byte_len)
        }
    }

    #[inline]
    fn header() -> Term {
        let arityval = word_size_of::<Self>();
        Term::make_header(arityval, Term::FLAG_SUBBINARY)
    }
}

unsafe impl AsTerm for SubBinary {
    unsafe fn as_term(&self) -> Term {
        Term::make_boxed(self as *const Self)
    }
}

impl Bitstring for SubBinary {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            let (_header, _flags, ptr, size) = self.to_raw_parts();
            slice::from_raw_parts(ptr, size.into())
        }
    }

    #[inline]
    fn full_byte_len(&self) -> usize {
        self.full_byte_len
    }

    #[inline]
    fn partial_byte_bit_len(&self) -> u8 {
        self.partial_byte_bit_len
    }

    #[inline]
    fn total_bit_len(&self) -> usize {
        self.full_byte_len * 8 + (self.partial_byte_bit_len as usize)
    }

    fn total_byte_len(&self) -> usize {
        self.full_byte_len + if self.is_binary() { 0 } else { 1 }
    }
}

impl CloneToProcess for SubBinary {
    fn clone_to_heap<A: HeapAlloc>(&self, heap: &mut A) -> Result<Term, AllocErr> {
        let real_bin_ptr = follow_moved(self.original).boxed_val();
        let real_bin = unsafe { *real_bin_ptr };
        // For ref-counted binaries and those that are already on the process heap,
        // we just need to copy the sub binary header, not the binary as well
        if real_bin.is_procbin() || (real_bin.is_heapbin() && heap.is_owner(real_bin_ptr)) {
            let size = mem::size_of::<Self>();
            unsafe {
                // Allocate space for header and copy it
                let ptr = heap.alloc(to_word_size(size))?.as_ptr() as *mut Self;
                ptr::copy_nonoverlapping(self as *const Self, ptr, size);
                Ok(Term::make_boxed(ptr))
            }
        } else {
            assert!(real_bin.is_heapbin());
            // Need to make sure that the heapbin is cloned as well, and that the header is suitably
            // updated
            let bin = unsafe { &*(real_bin_ptr as *mut HeapBin) };
            let new_bin = bin.clone_to_heap(heap)?;
            let size = mem::size_of::<Self>();
            unsafe {
                // Allocate space for header
                let ptr = heap.alloc(to_word_size(size))?.as_ptr() as *mut Self;
                // Write header, with modifications
                ptr::write(
                    ptr,
                    Self {
                        header: self.header,
                        original: new_bin.into(),
                        byte_offset: self.byte_offset,
                        bit_offset: self.bit_offset,
                        full_byte_len: self.full_byte_len,
                        partial_byte_bit_len: self.partial_byte_bit_len,
                        writable: self.writable,
                    },
                );

                Ok(Term::make_boxed(ptr))
            }
        }
    }
}

impl fmt::Debug for SubBinary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SubBinary")
            .field("header", &self.header.as_usize())
            .field("original", &self.original)
            .field("full_byte_len", &self.full_byte_len)
            .field("byte_offset", &self.byte_offset)
            .field("partial_byte_bit_len", &self.partial_byte_bit_len)
            .field("bit_offset", &self.bit_offset)
            .field("writable", &self.writable)
            .finish()
    }
}

impl Hash for SubBinary {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.byte_iter() {
            byte.hash(state);
        }

        for bit in self.partial_byte_bit_iter() {
            bit.hash(state);
        }
    }
}

impl<B: Bitstring> PartialEq<B> for SubBinary {
    #[inline]
    fn eq(&self, other: &B) -> bool {
        self.as_bytes().eq(other.as_bytes())
    }
}
impl Eq for SubBinary {}
impl<B: Bitstring> PartialOrd<B> for SubBinary {
    #[inline]
    fn partial_cmp(&self, other: &B) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl TryFrom<Term> for SubBinary {
    type Error = TypeError;

    fn try_from(term: Term) -> Result<Self, Self::Error> {
        term.to_typed_term().unwrap().try_into()
    }
}

impl TryFrom<TypedTerm> for SubBinary {
    type Error = TypeError;

    fn try_from(typed_term: TypedTerm) -> Result<Self, Self::Error> {
        match typed_term {
            TypedTerm::SubBinary(subbinary) => Ok(subbinary),
            _ => Err(TypeError),
        }
    }
}

impl TryInto<String> for SubBinary {
    type Error = runtime::Exception;

    fn try_into(self) -> Result<String, Self::Error> {
        if self.is_binary() {
            if self.is_aligned() {
                match str::from_utf8(self.as_bytes()) {
                    Ok(s) => Ok(s.to_owned()),
                    Err(_) => Err(badarg!()),
                }
            } else {
                let byte_vec: Vec<u8> = self.byte_iter().collect();

                String::from_utf8(byte_vec).map_err(|_| badarg!())
            }
        } else {
            Err(badarg!())
        }
    }
}