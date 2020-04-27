#![allow(clippy::new_ret_no_self)]

use std::{
    cell::{Cell, UnsafeCell},
    collections::HashMap,
    path::PathBuf,
    str,
};

use rboehm::Gc;

use crate::vm::{
    core::VM,
    error::{VMError, VMErrorKind},
    objects::{Method, Obj, ObjType, StaticObjType},
    val::{NotUnboxable, Val, ValKind},
};

#[derive(Debug)]
pub struct Class {
    metacls: Cell<Val>,
    pub name: Val,
    pub path: PathBuf,
    /// Offset to this class's instructions in VM::instrs.
    pub instrs_off: usize,
    supercls: Cell<Val>,
    pub num_inst_vars: usize,
    pub methods: HashMap<String, Gc<Method>>,
    inst_vars: UnsafeCell<Vec<Val>>,
}

impl Obj for Class {
    fn dyn_objtype(&self) -> ObjType {
        ObjType::Class
    }

    fn get_class(&self, _: &mut VM) -> Val {
        debug_assert!(self.metacls.get().valkind() != ValKind::ILLEGAL);
        self.metacls.get()
    }

    fn inst_var_lookup(&self, n: usize) -> Val {
        let inst_vars = unsafe { &mut *self.inst_vars.get() };
        inst_vars[n]
    }

    fn inst_var_set(&self, n: usize, v: Val) {
        let inst_vars = unsafe { &mut *self.inst_vars.get() };
        inst_vars[n] = v;
    }
}

impl NotUnboxable for Class {}

impl StaticObjType for Class {
    fn static_objtype() -> ObjType {
        ObjType::Class
    }
}

impl Class {
    pub fn new(
        vm: &VM,
        metacls: Val,
        name: Val,
        path: PathBuf,
        instrs_off: usize,
        supercls: Val,
        num_inst_vars: usize,
        methods: HashMap<String, Gc<Method>>,
    ) -> Self {
        let cls = Class {
            metacls: Cell::new(metacls),
            name,
            path,
            instrs_off,
            supercls: Cell::new(supercls),
            num_inst_vars,
            methods,
            inst_vars: UnsafeCell::new(vec![]),
        };
        cls.set_metacls(vm, metacls);
        cls
    }

    pub fn name(&self, _: &VM) -> Result<Val, Box<VMError>> {
        Ok(self.name)
    }

    pub fn get_method(&self, vm: &VM, msg: &str) -> Result<Gc<Method>, Box<VMError>> {
        self.methods.get(msg).map(|x| Ok(*x)).unwrap_or_else(|| {
            let supercls = self.supercls(vm);
            if supercls != vm.nil {
                supercls.downcast::<Class>(vm)?.get_method(vm, msg)
            } else {
                Err(VMError::new(vm, VMErrorKind::UnknownMethod(msg.to_owned())))
            }
        })
    }

    pub fn set_metacls(&self, vm: &VM, cls_val: Val) {
        // This method is called during VM bootstrapping when not all objects have valid
        // references.
        if cls_val.valkind() != ValKind::ILLEGAL {
            let cls: &Class = cls_val.downcast(vm).unwrap();
            let mut inst_vars = Vec::with_capacity(cls.num_inst_vars);
            inst_vars.resize(cls.num_inst_vars, Val::illegal());
            self.metacls.set(cls_val);
            *unsafe { &mut *self.inst_vars.get() } = inst_vars;
        }
    }

    pub fn supercls(&self, _: &VM) -> Val {
        self.supercls.get()
    }

    pub fn set_supercls(&self, _: &VM, cls: Val) {
        self.supercls.set(cls);
    }
}
