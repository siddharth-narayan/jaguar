cd ./amdgpu
cargo clean && cargo build --profile release && mv ../target/amdgcn-amd-amdhsa/release/jaguar_amdgpu.elf ../jaguar/src

cd ../jaguar
cargo build --profile release
llvm-objdump --disassemble-all  src/jaguar_amdgpu.elf > disassembly

rocprofv3 --att --att-buffer-size 0x80000000 -d trace -- ../target/release/jaguar > out
rocprof-compute-viewer