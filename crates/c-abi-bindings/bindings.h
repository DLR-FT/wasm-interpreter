#include <stdint.h>

/**
 * Error cause for a failed operation
 */
typedef enum CError {
  /**
   * Validation failed
   */
  ValidationError,
  /**
   * The validation info is invalid, i.e. a nullptr
   */
  InvalidValidationInfo,
  /**
   * Instantiation failed
   */
  InstantiationError,
  /**
   * The store is invalid, i.e. a nullptr
   */
  InvalidStore,
  UTF8Error,
  /**
   * A C-str is not null terminated
   */
  CStrNotNullTerminated,
  NoSuchExport,
  ExternAddrIsNotAFunc,
  RuntimeError,
} CError;

/**
 * Opaque wrapper for the external address of a Wasm module's export. Use this to get a handle i.e.
 * to an function in the Wasm module.
 */
typedef struct ExternAddr ExternAddr;

/**
 * Opaque wrapper for a Wasm module. Stores validation information and a reference to the
 * corresponding bytes of the bytecode making up a Wasm module.
 */
typedef struct Module Module;

/**
 * Opaque wrapper for a Wasm module instantion outcome. Stores the module instance's address and
 * its exports.
 */
typedef struct ModuleInst ModuleInst;

/**
 * Opaque wrapper for a Wasm store. Stores the state of an interpreter instance.
 */
typedef struct Store Store;

/**
 * Result of an operation; either success (`Ok`) or an error
 */
typedef enum CResult_____Store_Tag {
  Ok_____Store,
  Err_____Store,
} CResult_____Store_Tag;

typedef struct CResult_____Store {
  CResult_____Store_Tag tag;
  union {
    struct {
      struct Store *ok;
    };
    struct {
      enum CError err;
    };
  };
} CResult_____Store;

/**
 * Type for user data, just a void pointer, to allow arbitrary context to be associated with the
 * interpreter.
 */
typedef void *UserDataType;

/**
 * Result of an operation; either success (`Ok`) or an error
 */
typedef enum CResult_____Module_Tag {
  Ok_____Module,
  Err_____Module,
} CResult_____Module_Tag;

typedef struct CResult_____Module {
  CResult_____Module_Tag tag;
  union {
    struct {
      struct Module *ok;
    };
    struct {
      enum CError err;
    };
  };
} CResult_____Module;

/**
 * Result of an operation; either success (`Ok`) or an error
 */
typedef enum CResult_____ModuleInst_Tag {
  Ok_____ModuleInst,
  Err_____ModuleInst,
} CResult_____ModuleInst_Tag;

typedef struct CResult_____ModuleInst {
  CResult_____ModuleInst_Tag tag;
  union {
    struct {
      struct ModuleInst *ok;
    };
    struct {
      enum CError err;
    };
  };
} CResult_____ModuleInst;

/**
 * Result of an operation; either success (`Ok`) or an error
 */
typedef enum CResult_____ExternAddr_Tag {
  Ok_____ExternAddr,
  Err_____ExternAddr,
} CResult_____ExternAddr_Tag;

typedef struct CResult_____ExternAddr {
  CResult_____ExternAddr_Tag tag;
  union {
    struct {
      struct ExternAddr *ok;
    };
    struct {
      enum CError err;
    };
  };
} CResult_____ExternAddr;

/**
 * Initialize a new store. See
 * <https://webassembly.github.io/spec/core/appendix/embedding.html#mathrm-store-init-xref-exec-runtime-syntax-store-mathit-store> for background.
 *
 * # Safety
 *
 * - Ensure that `new_store` is a valid pointer to an opaque pointer.
 * - Ensure that `new_store` is only ever used if the return value was `CResult::Ok`.
 */
struct CResult_____Store store_init(UserDataType user_data);

/**
 * Drop a store.
 *
 * This deallocates a [`Store`] on the heap.
 *
 * # Safety
 *
 * - Ensure that `store` is a valid pointer to an initialized store.
 */
CResult store_drop(struct Store *store);

/**
 * Decode/parse and validate Wasm bytecode.
 *
 * # Safety
 *
 * - Ensure that `wasm_bytecode` is a pointer living at least until when the resulting module is
 *   freed.
 * - Ensure that `new_module` is a valid pointer to an opaque pointer.
 * - Ensure that `new_store` is only ever used if the return value was `CResult::Ok`.
 */
struct CResult_____Module module_validate(const uint8_t *wasm_bytecode,
                                          uintptr_t wasm_bytecode_length);

/**
 * Drop a module.
 *
 * This deallocates a [`Module`] on the heap.
 *
 * # Safety
 *
 * - Ensure that `module` is a valid pointer to an initialized module.
 */
CResult module_drop(struct Module *module);

/**
 * Instantiate a module into a store.
 *
 * # Safety
 *
 * - Ensure that `store` is a valid pointer to an intialized store.
 * - Ensure that `module` is a valid pointer to a validated module.
 */
struct CResult_____ModuleInst module_instantiate(struct Store *store, struct Module *module);

/**
 * Drop a module instance.
 *
 * This deallocates a [`ModuleInstance`] on the heap.
 *
 * # Safety
 *
 * - Ensure that `module_inst` is a valid pointer to an initialized module.
 */
CResult module_inst_drop(struct ModuleInst *module_inst);

/**
 * # Safety
 *
 * Ensure that name is a pointer to a c_char array being, that is null terminated and at least
 * `name_length` chars long.
 */
struct CResult_____ExternAddr instance_export(const struct ModuleInst *module_inst,
                                              const unsigned char *name,
                                              uintptr_t name_length);

CResult func_invoke(struct Store *store, const struct ExternAddr *extern_addr, uint64_t fuel);
