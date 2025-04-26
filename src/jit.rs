use crate::vm;
use cranelift::jit::JITBuilder;
use cranelift::jit::JITModule;
use cranelift::prelude::*;
use cranelift_module::DataDescription;
use cranelift_module::Module;
use cranelift_module::Linkage;

pub struct JIT {
    code: Vec<vm::Op>,
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    data_description: DataDescription,
    module: JITModule,
}

impl JIT {
    pub fn new(code: Vec<vm::Op>) -> Self {        
        let builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap_or_else(|msg| {
            panic!("Failed to create JITBuilder: {}", msg);
        });
        
        
        let module = JITModule::new(builder);

        Self { 
            code: code,
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module: module,
        }
    }
}

impl JIT {

    // The final function will be called with the following signature:
    // fn(tape_ptr: *mut u8,
    //    rt_ptr: *mut u8,
    //    write_char: extern "C" fn(rt_ptr: *mut u8, u8),
    //    read_char: extern "C" fn(rt_ptr: *mut u8) -> u8)
    //    -> u8
    // For now the tape is not growable, so we can just pass a pointer to the tape

    pub fn compile(&mut self) -> Result<*const u8, String> {
        // Translate the IR into self.ctx.func
        self.translate()?;
        // Declare the JIT function using the signature we built
        let func_id = self.module
            .declare_function("bf_jit", Linkage::Export, &self.ctx.func.signature)
            .map_err(|e| e.to_string())?;
        // Define and finalize
        self.module.define_function(func_id, &mut self.ctx).map_err(|e| e.to_string())?;
        self.module.clear_context(&mut self.ctx);
        self.module.finalize_definitions().unwrap();
        // Return the generated code pointer
        Ok(self.module.get_finalized_function(func_id))
    }
    

    fn translate(&mut self) -> Result<(), String> {
        // Build function signature for BF JIT
        let ptr_ty = self.module.target_config().pointer_type();
        // params: tape ptr, runtime ptr, write_fn ptr, read_fn ptr
        self.ctx.func.signature.params.push(AbiParam::new(ptr_ty)); // tape ptr
        self.ctx.func.signature.params.push(AbiParam::new(ptr_ty)); // runtime ptr
        self.ctx.func.signature.params.push(AbiParam::new(ptr_ty)); // write fn ptr
        self.ctx.func.signature.params.push(AbiParam::new(ptr_ty)); // read fn ptr
        self.ctx.func.signature.returns.push(AbiParam::new(types::I8));

        let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

        // Create a block for each instruction plus an exit block
        println!("Translate JIT: code.len()={}", self.code.len());
        let num_ops = self.code.len();
        let mut blocks = Vec::with_capacity(num_ops + 1);
        println!("Translate JIT: creating {} blocks (op count + exit)", num_ops + 1);
        for _ in 0..(num_ops + 1) {
            blocks.push(builder.create_block());
        }
        // Entry block
        let entry_block = blocks[0];
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        //println!("Switched to entry block={:?}", entry_block);

        // Function parameters
        let tape_ptr = builder.block_params(entry_block)[0];
        let rt_ptr   = builder.block_params(entry_block)[1];
        let write_ptr= builder.block_params(entry_block)[2];
        let read_ptr = builder.block_params(entry_block)[3];

        // Prepare signatures for indirect calls: both take runtime ptr first
        let write_sig_ref = builder.import_signature({
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(ptr_ty));    // runtime ptr
            sig.params.push(AbiParam::new(types::I8)); // byte to write
            sig
        });
        let read_sig_ref = builder.import_signature({
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(ptr_ty));    // runtime ptr
            sig.returns.push(AbiParam::new(types::I8)); // byte read
            sig
        });

        // Variable for current cell pointer
        let cell_ptr = Variable::new(0);
        builder.declare_var(cell_ptr, ptr_ty);
        builder.def_var(cell_ptr, tape_ptr);

        // Translate each Brainfuck operation
        for (i, op) in self.code.iter().enumerate() {
            let cur = blocks[i];
            let nxt = blocks[i + 1];
            builder.switch_to_block(cur);
            //println!("Switching to block {} ({:?}) for op {:?}", i, cur, op);

            let cell_addr = builder.use_var(cell_ptr);
            match op {
                vm::Op::Inc(n) => {
                    let v = builder.ins().load(types::I8, MemFlags::new(), cell_addr, 0);
                    let inc = builder.ins().iconst(types::I8, *n as i64);
                    let res = builder.ins().iadd(v, inc);
                    builder.ins().store(MemFlags::new(), res, cell_addr, 0);
                    builder.ins().jump(nxt, &[]);
                }
                vm::Op::Dec(n) => {
                    let v = builder.ins().load(types::I8, MemFlags::new(), cell_addr, 0);
                    let dec = builder.ins().iconst(types::I8, *n as i64);
                    let res = builder.ins().isub(v, dec);
                    builder.ins().store(MemFlags::new(), res, cell_addr, 0);
                    builder.ins().jump(nxt, &[]);
                }
                vm::Op::MovR(n) => {
                    let ptr = builder.use_var(cell_ptr);
                    let ofs = builder.ins().iconst(ptr_ty, *n as i64);
                    let newp = builder.ins().iadd(ptr, ofs);
                    builder.def_var(cell_ptr, newp);
                    builder.ins().jump(nxt, &[]);
                }
                vm::Op::MovL(n) => {
                    let ptr = builder.use_var(cell_ptr);
                    let ofs = builder.ins().iconst(ptr_ty, *n as i64);
                    let newp = builder.ins().isub(ptr, ofs);
                    builder.def_var(cell_ptr, newp);
                    builder.ins().jump(nxt, &[]);
                }
                vm::Op::JmpIfZ(target) => {
                    let v = builder.ins().load(types::I8, MemFlags::new(), cell_addr, 0);
                    let zv = builder.ins().iconst(types::I8, 0);
                    let cmp = builder.ins().icmp(IntCC::Equal, v, zv);
                    builder.ins().brif(cmp, blocks[*target as usize], &[], nxt, &[]);
                }
                vm::Op::JmpIfNZ(target) => {
                    let v = builder.ins().load(types::I8, MemFlags::new(), cell_addr, 0);
                    let zv = builder.ins().iconst(types::I8, 0);
                    let cmp = builder.ins().icmp(IntCC::NotEqual, v, zv);
                    builder.ins().brif(cmp, blocks[*target as usize], &[], nxt, &[]);
                }
                vm::Op::Print => {
                    let v = builder.ins().load(types::I8, MemFlags::new(), cell_addr, 0);
                    builder.ins().call_indirect(write_sig_ref, write_ptr, &[rt_ptr, v]);
                    builder.ins().jump(nxt, &[]);
                }
                vm::Op::Read => {
                    let call = builder.ins().call_indirect(read_sig_ref, read_ptr, &[rt_ptr]);
                    let rv = builder.inst_results(call)[0];
                    builder.ins().store(MemFlags::new(), rv, cell_addr, 0);
                    builder.ins().jump(nxt, &[]);
                }
                vm::Op::Nop => {
                    builder.ins().jump(nxt, &[]);
                }
            }
        }

        // Exit block
        let exit_b = blocks[self.code.len()];
        //println!("Switching to exit block {} ({:?})", self.code.len(), exit_b);
        builder.switch_to_block(exit_b);
        //println!("Translating exit block {} ({:?})", self.code.len(), exit_b);
        let final_addr = builder.use_var(cell_ptr);
        let final_v = builder.ins().load(types::I8, MemFlags::new(), final_addr, 0);
        builder.ins().return_(&[final_v]);
        // Seal all blocks now that all edges are created
        builder.seal_all_blocks();
        builder.finalize();

        Ok(())
    }
}

