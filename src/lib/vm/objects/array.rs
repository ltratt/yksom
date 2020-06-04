#![allow(clippy::new_ret_no_self)]

use std::cell::UnsafeCell;

use crate::vm::{
    core::VM,
    error::{VMError, VMErrorKind},
    objects::{Obj, ObjType, StaticObjType},
    val::{NotUnboxable, Val},
};

#[derive(Debug)]
pub struct NormalArray {
    store: UnsafeCell<Vec<Val>>,
}

impl Obj for NormalArray {
    fn dyn_objtype(&self) -> ObjType {
        ObjType::Array
    }

    fn get_class(&self, vm: &mut VM) -> Val {
        vm.array_cls
    }

    fn length(&self) -> usize {
        let store = unsafe { &*self.store.get() };
        store.len()
    }
}

impl NotUnboxable for NormalArray {}

impl StaticObjType for NormalArray {
    fn static_objtype() -> ObjType {
        ObjType::Array
    }
}

impl NormalArray {
    pub fn new(vm: &mut VM, len: usize) -> Val {
        let mut store = Vec::with_capacity(len);
        store.resize(len, vm.nil);
        Val::from_obj(
            vm,
            NormalArray {
                store: UnsafeCell::new(store),
            },
        )
    }

    pub fn from_vec(vm: &mut VM, store: Vec<Val>) -> Val {
        Val::from_obj(
            vm,
            NormalArray {
                store: UnsafeCell::new(store),
            },
        )
    }

    /// Return the item at index `idx` (using SOM indexing starting at 1) or an error if the index
    /// is invalid.
    pub fn at(&self, vm: &VM, mut idx: usize) -> Result<Val, Box<VMError>> {
        let store = unsafe { &*self.store.get() };
        if idx > 0 && idx <= store.len() {
            idx -= 1;
            Ok(*unsafe { store.get_unchecked(idx) })
        } else {
            Err(VMError::new(
                vm,
                VMErrorKind::IndexError {
                    tried: idx,
                    max: store.len(),
                },
            ))
        }
    }

    /// Return the item at index `idx` (using SOM indexing starting at 1). This will lead to
    /// undefined behaviour if the index is invalid.
    pub unsafe fn unchecked_at(&self, mut idx: usize) -> Val {
        debug_assert!(idx > 0);
        let store = &*self.store.get();
        debug_assert!(idx <= store.len());
        idx -= 1;
        *store.get_unchecked(idx)
    }

    /// Set the item at index `idx` (using SOM indexing starting at 1) to `val` or return an error
    /// if the index is invalid.
    pub fn at_put(&self, vm: &VM, mut idx: usize, val: Val) -> Result<(), Box<VMError>> {
        let store = unsafe { &mut *self.store.get() };
        if idx > 0 && idx <= store.len() {
            idx -= 1;
            *unsafe { store.get_unchecked_mut(idx) } = val;
            Ok(())
        } else {
            Err(VMError::new(
                vm,
                VMErrorKind::IndexError {
                    tried: idx,
                    max: store.len(),
                },
            ))
        }
    }

    /// Iterate over this array's values.
    pub fn iter<'a>(&'a self) -> NormalArrayIterator<'a> {
        NormalArrayIterator { arr: self, i: 0 }
    }
}

pub struct NormalArrayIterator<'a> {
    arr: &'a NormalArray,
    i: usize,
}

impl<'a> Iterator for NormalArrayIterator<'a> {
    type Item = Val;

    fn next(&mut self) -> Option<Val> {
        if self.i < self.arr.length() {
            self.i += 1;
            Some(unsafe { self.arr.unchecked_at(self.i) })
        } else {
            None
        }
    }
}
