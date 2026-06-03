#include <stdint.h>

/// Error cause for a failed operation
enum class CError {
  /// Validation failed
  ValidationError,
  /// The validation info is invalid, i.e. a nullptr
  InvalidValidationInfo,
  /// Instantiation failed
  InstantiationError,
  /// The store is invalid, i.e. a nullptr
  InvalidStore,
  UTF8Error,
  /// A C-str is not null terminated
  CStrNotNullTerminated,
  NoSuchExport,
  ExternAddrIsNotAFunc,
  RuntimeError,
};

/// Opaque wrapper for the external address of a Wasm module's export. Use this to get a handle i.e.
/// to an function in the Wasm module.
struct ExternAddr;

/// Opaque wrapper for a Wasm module. Stores validation information and a reference to the
/// corresponding bytes of the bytecode making up a Wasm module.
struct Module;

/// Opaque wrapper for a Wasm module instantion outcome. Stores the module instance's address and
/// its exports.
struct ModuleInst;

/// Opaque wrapper for a Wasm store. Stores the state of an interpreter instance.
struct Store;

/// Result of an operation; either success (`Ok`) or an error
template<typename T>
struct CResult {
  enum class Tag {
    Ok,
    Err,
  };

  struct Ok_Body {
    T _0;
  };

  struct Err_Body {
    CError _0;
  };

  Tag tag;
  union {
    Ok_Body ok;
    Err_Body err;
  };
};

/// Type for user data, just a void pointer, to allow arbitrary context to be associated with the
/// interpreter.
using UserDataType = void*;

extern "C" {

/// Initialize a new store. See
/// <https://webassembly.github.io/spec/core/appendix/embedding.html#mathrm-store-init-xref-exec-runtime-syntax-store-mathit-store> for background.
///
/// # Safety
///
/// - Ensure that `new_store` is a valid pointer to an opaque pointer.
/// - Ensure that `new_store` is only ever used if the return value was `CResult::Ok`.
CResult<Store*> store_init(UserDataType user_data);

/// Drop a store.
///
/// This deallocates a [`Store`] on the heap.
///
/// # Safety
///
/// - Ensure that `store` is a valid pointer to an initialized store.
CResult store_drop(Store *store);

/// Decode/parse and validate Wasm bytecode.
///
/// # Safety
///
/// - Ensure that `wasm_bytecode` is a pointer living at least until when the resulting module is
///   freed.
/// - Ensure that `new_module` is a valid pointer to an opaque pointer.
/// - Ensure that `new_store` is only ever used if the return value was `CResult::Ok`.
CResult<Module*> module_validate(const uint8_t *wasm_bytecode, uintptr_t wasm_bytecode_length);

/// Drop a module.
///
/// This deallocates a [`Module`] on the heap.
///
/// # Safety
///
/// - Ensure that `module` is a valid pointer to an initialized module.
CResult module_drop(Module *module);

/// Instantiate a module into a store.
///
/// # Safety
///
/// - Ensure that `store` is a valid pointer to an intialized store.
/// - Ensure that `module` is a valid pointer to a validated module.
CResult<ModuleInst*> module_instantiate(Store *store, Module *module);

/// Drop a module instance.
///
/// This deallocates a [`ModuleInstance`] on the heap.
///
/// # Safety
///
/// - Ensure that `module_inst` is a valid pointer to an initialized module.
CResult module_inst_drop(ModuleInst *module_inst);

/// # Safety
///
/// Ensure that name is a pointer to a c_char array being, that is null terminated and at least
/// `name_length` chars long.
CResult<ExternAddr*> instance_export(const ModuleInst *module_inst,
                                     const unsigned char *name,
                                     uintptr_t name_length);

CResult func_invoke(Store *store, const ExternAddr *extern_addr, uint64_t fuel);

}  // extern "C"
