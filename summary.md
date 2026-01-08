
  Summary: aarch64 Cranelift Boot Test

  | Component        | Status                         |
  |------------------|--------------------------------|
  | Cranelift kernel | ✅ Boots successfully          |
  | LLVM initfs      | ✅ Works with Cranelift kernel |
  | virtio-blkd      | ✅ Works (legacy INTx#)        |
  | RedoxFS          | ✅ Mounts                      |
  | Login prompt     | ✅ Reached                     |
  | fbcond           | ❌ Crashes (userspace bug)     |
  | virtio-netd      | ❌ Needs INTx# fallback        |

  The server-official.iso already has the Cranelift kernel and boots to login! The desktop/redox-live.iso was corrupted during previous modifications.
